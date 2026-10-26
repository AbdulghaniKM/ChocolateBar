use std::thread;

use crossbeam_channel::Sender;
use windows::Devices::Bluetooth::Rfcomm::RfcommServiceId;
use windows::Devices::Bluetooth::{BluetoothConnectionStatus, BluetoothDevice};
use windows::Devices::Enumeration::DeviceInformation;
use windows::Networking::Sockets::StreamSocket;
use windows::Storage::Streams::{DataReader, DataWriter, InputStreamOptions};
use windows_core::{Error, GUID, HSTRING};

use super::model::{is_galaxy_buds, BudsModel};

const READ_CHUNK_BYTES: u32 = 1024;
const SHARING_VIOLATION: u32 = 0x8007_0020;
const ACCESS_DENIED: u32 = 0x8007_0005;
const ADDRESS_IN_USE: u32 = 0x8007_2740;
const CONNECTION_REFUSED: u32 = 0x8007_274D;

pub struct PairedBuds {
    pub device: BluetoothDevice,
    pub device_id: String,
    pub address: String,
    pub name: String,
    pub model: BudsModel,
}

impl PairedBuds {
    pub fn is_connected(&self) -> bool {
        self.device.ConnectionStatus().is_ok_and(|status| status == BluetoothConnectionStatus::Connected)
    }
}

pub enum OpenError {
    InUse,
    Failed(String),
}

impl OpenError {
    fn from_windows(error: &Error) -> Self {
        match error.code().0 as u32 {
            SHARING_VIOLATION | ACCESS_DENIED | ADDRESS_IN_USE | CONNECTION_REFUSED => Self::InUse,
            _ => Self::Failed(error.to_string()),
        }
    }
}

pub enum LinkEvent {
    Data(Vec<u8>),
    Closed,
}

pub struct BudsLink {
    socket: StreamSocket,
    writer: DataWriter,
    reader: DataReader,
}

impl BudsLink {
    pub fn send(&self, packet: &[u8]) -> windows_core::Result<()> {
        self.writer.WriteBytes(packet)?;
        self.writer.StoreAsync()?.join()?;
        Ok(())
    }

    pub fn spawn_reader(&self, events: Sender<LinkEvent>) {
        let reader = self.reader.clone();
        let spawned = thread::Builder::new().name("buds-rfcomm-read".into()).spawn(move || {
            while let Some(bytes) = read_chunk(&reader) {
                if events.send(LinkEvent::Data(bytes)).is_err() {
                    return;
                }
            }
            let _ = events.send(LinkEvent::Closed);
        });
        if let Err(error) = spawned {
            log::warn!("buds: reader thread failed to spawn: {error}");
        }
    }
}

impl Drop for BudsLink {
    fn drop(&mut self) {
        let _ = self.socket.Close();
    }
}

pub fn find_paired_buds() -> windows_core::Result<Vec<PairedBuds>> {
    let selector = BluetoothDevice::GetDeviceSelectorFromPairingState(true)?;
    let candidates = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.join()?;
    let mut found = Vec::new();
    for index in 0..candidates.Size()? {
        let information = candidates.GetAt(index)?;
        let name = information.Name()?.to_string();
        if !is_galaxy_buds(&name) {
            continue;
        }
        if let Some(buds) = load_paired_buds(&information.Id()?, name) {
            found.push(buds);
        }
    }
    Ok(found)
}

pub fn open_link(buds: &PairedBuds) -> Result<BudsLink, OpenError> {
    let mut last_error = OpenError::Failed("no RFCOMM service found".into());
    for uuid in buds.model.service_uuids() {
        match connect_service(&buds.device, uuid) {
            Ok(Some(link)) => return Ok(link),
            Ok(None) => {}
            Err(error) => last_error = OpenError::from_windows(&error),
        }
    }
    Err(last_error)
}

fn load_paired_buds(device_id: &HSTRING, name: String) -> Option<PairedBuds> {
    let device = BluetoothDevice::FromIdAsync(device_id).ok()?.join().ok()?;
    let address = format!("{:012x}", device.BluetoothAddress().ok()?);
    let model = BudsModel::from_device_name(&name);
    Some(PairedBuds { device, device_id: device_id.to_string(), address, name, model })
}

fn connect_service(device: &BluetoothDevice, uuid: GUID) -> windows_core::Result<Option<BudsLink>> {
    let service_id = RfcommServiceId::FromUuid(uuid)?;
    let services = device.GetRfcommServicesForIdAsync(&service_id)?.join()?.Services()?;
    if services.Size()? == 0 {
        return Ok(None);
    }
    let service = services.GetAt(0)?;
    let socket = StreamSocket::new()?;
    socket.ConnectAsync(&service.ConnectionHostName()?, &service.ConnectionServiceName()?)?.join()?;
    let reader = DataReader::CreateDataReader(&socket.InputStream()?)?;
    reader.SetInputStreamOptions(InputStreamOptions::Partial)?;
    let writer = DataWriter::CreateDataWriter(&socket.OutputStream()?)?;
    Ok(Some(BudsLink { socket, writer, reader }))
}

fn read_chunk(reader: &DataReader) -> Option<Vec<u8>> {
    let loaded = reader.LoadAsync(READ_CHUNK_BYTES).ok()?.join().ok()?;
    if loaded == 0 {
        return None;
    }
    let mut bytes = vec![0u8; loaded as usize];
    reader.ReadBytes(&mut bytes).ok()?;
    Some(bytes)
}
