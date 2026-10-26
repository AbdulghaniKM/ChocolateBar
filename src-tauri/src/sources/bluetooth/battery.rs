use std::collections::HashMap;
use std::thread;

use crossbeam_channel::{bounded, Sender};
use windows::Devices::Bluetooth::BluetoothLEDevice;
use windows::Devices::Bluetooth::GenericAttributeProfile::{
    GattCharacteristic, GattClientCharacteristicConfigurationDescriptorValue, GattCommunicationStatus,
    GattValueChangedEventArgs,
};
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationKind};
use windows::Foundation::TypedEventHandler;
use windows::Storage::Streams::{DataReader, IBuffer};
use windows::Win32::Foundation::E_FAIL;
use windows_core::{Result, GUID, HSTRING};

use super::properties::{requested_property_keys, PropertyPatch};
use super::watcher::DeviceMessage;

const BATTERY_SERVICE: GUID = GUID::from_u128(0x0000180f_0000_1000_8000_00805f9b34fb);
const BATTERY_LEVEL: GUID = GUID::from_u128(0x00002a19_0000_1000_8000_00805f9b34fb);

pub fn read_classic_battery(device_id: &str) -> Option<u8> {
    classic_battery_percent(device_id)
        .inspect_err(|error| log::debug!("bluetooth: classic battery unavailable for {device_id}: {error}"))
        .ok()
        .flatten()
}

fn classic_battery_percent(device_id: &str) -> Result<Option<u8>> {
    let information = DeviceInformation::CreateFromIdAsyncWithKindAndAdditionalProperties(
        &HSTRING::from(device_id),
        &requested_property_keys!(),
        DeviceInformationKind::AssociationEndpoint,
    )?
    .join()?;
    let properties = information.Properties()?;
    Ok(PropertyPatch::from_lookup(|key| properties.Lookup(key).ok()).battery)
}

#[derive(Default)]
pub struct LowEnergyBatteryWatchers {
    stop_signals: HashMap<String, Sender<()>>,
}

impl LowEnergyBatteryWatchers {
    pub fn is_watching(&self, device_id: &str) -> bool {
        self.stop_signals.contains_key(device_id)
    }

    pub fn watch(&mut self, device_id: String, messages: Sender<DeviceMessage>) {
        let (stop, stopped) = bounded::<()>(1);
        let id = device_id.clone();
        let spawned = thread::Builder::new().name("bluetooth-ble-battery".into()).spawn(move || {
            match attach_battery_notifications(&id, &messages) {
                Ok(subscription) => {
                    let _ = stopped.recv();
                    drop(subscription);
                }
                Err(error) => log::debug!("bluetooth: no BLE battery for {id}: {error}"),
            }
        });
        match spawned {
            Ok(_) => {
                self.stop_signals.insert(device_id, stop);
            }
            Err(error) => log::warn!("bluetooth: BLE battery worker failed to spawn: {error}"),
        }
    }

    pub fn stop(&mut self, device_id: &str) {
        self.stop_signals.remove(device_id);
    }
}

struct BatterySubscription {
    characteristic: GattCharacteristic,
    token: i64,
    _device: BluetoothLEDevice,
}

impl Drop for BatterySubscription {
    fn drop(&mut self) {
        let _ = self.characteristic.RemoveValueChanged(self.token);
    }
}

fn attach_battery_notifications(device_id: &str, messages: &Sender<DeviceMessage>) -> Result<BatterySubscription> {
    let device = BluetoothLEDevice::FromIdAsync(&HSTRING::from(device_id))?.join()?;
    let characteristic = find_battery_characteristic(&device)?;
    let reading = characteristic.ReadValueAsync()?.join()?;
    send_percent(messages, device_id, &reading.Value()?);
    let notifying_id = device_id.to_owned();
    let notifying_messages = messages.clone();
    let handler = TypedEventHandler::<GattCharacteristic, GattValueChangedEventArgs>::new(move |_, args| {
        send_percent(&notifying_messages, &notifying_id, &args.ok()?.CharacteristicValue()?);
        Ok(())
    });
    let token = characteristic.ValueChanged(&handler)?;
    characteristic
        .WriteClientCharacteristicConfigurationDescriptorAsync(GattClientCharacteristicConfigurationDescriptorValue::Notify)?
        .join()?;
    Ok(BatterySubscription { characteristic, token, _device: device })
}

fn find_battery_characteristic(device: &BluetoothLEDevice) -> Result<GattCharacteristic> {
    let services = device.GetGattServicesForUuidAsync(BATTERY_SERVICE)?.join()?;
    ensure_success(services.Status()?)?;
    let service = services.Services()?.GetAt(0)?;
    let characteristics = service.GetCharacteristicsForUuidAsync(BATTERY_LEVEL)?.join()?;
    ensure_success(characteristics.Status()?)?;
    characteristics.Characteristics()?.GetAt(0)
}

fn ensure_success(status: GattCommunicationStatus) -> Result<()> {
    if status == GattCommunicationStatus::Success {
        Ok(())
    } else {
        Err(E_FAIL.into())
    }
}

fn send_percent(messages: &Sender<DeviceMessage>, device_id: &str, buffer: &IBuffer) {
    let Some(percent) = read_percent(buffer).filter(|percent| *percent <= 100) else {
        return;
    };
    let _ = messages.send(DeviceMessage::Updated {
        id: device_id.to_owned(),
        patch: PropertyPatch::battery_only(percent),
    });
}

fn read_percent(buffer: &IBuffer) -> Option<u8> {
    DataReader::FromBuffer(buffer).ok()?.ReadByte().ok()
}
