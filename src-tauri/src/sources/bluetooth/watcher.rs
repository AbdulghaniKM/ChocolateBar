use crossbeam_channel::Sender;
use windows::Devices::Bluetooth::{BluetoothDevice, BluetoothLEDevice};
use windows::Devices::Enumeration::{
    DeviceInformation, DeviceInformationKind, DeviceInformationUpdate, DeviceWatcher,
};
use windows::Foundation::TypedEventHandler;
use windows_core::{Result, HSTRING};

use super::properties::{requested_property_keys, LinkKind, PropertyPatch};

#[derive(Debug)]
pub enum DeviceMessage {
    Added { id: String, kind: LinkKind, name: String, patch: PropertyPatch },
    Updated { id: String, patch: PropertyPatch },
    Removed { id: String },
    EnumerationCompleted,
}

pub const WATCHER_COUNT: usize = 2;

struct WatcherSubscription {
    watcher: DeviceWatcher,
    tokens: Vec<(fn(&DeviceWatcher, i64) -> Result<()>, i64)>,
}

impl Drop for WatcherSubscription {
    fn drop(&mut self) {
        let _ = self.watcher.Stop();
        for (remove, token) in &self.tokens {
            let _ = remove(&self.watcher, *token);
        }
    }
}

pub struct BluetoothWatchers {
    _subscriptions: Vec<WatcherSubscription>,
}

impl BluetoothWatchers {
    pub fn start(messages: &Sender<DeviceMessage>) -> Result<Self> {
        let classic = start_watcher(&BluetoothDevice::GetDeviceSelectorFromPairingState(true)?, LinkKind::Classic, messages)?;
        let low_energy =
            start_watcher(&BluetoothLEDevice::GetDeviceSelectorFromPairingState(true)?, LinkKind::LowEnergy, messages)?;
        Ok(Self { _subscriptions: vec![classic, low_energy] })
    }
}

fn start_watcher(selector: &HSTRING, kind: LinkKind, messages: &Sender<DeviceMessage>) -> Result<WatcherSubscription> {
    let watcher = DeviceInformation::CreateWatcherWithKindAqsFilterAndAdditionalProperties(
        selector,
        &requested_property_keys(),
        DeviceInformationKind::AssociationEndpoint,
    )?;
    let tokens = vec![
        (
            DeviceWatcher::RemoveAdded as fn(&DeviceWatcher, i64) -> Result<()>,
            watcher.Added(&added_handler(kind, messages.clone()))?,
        ),
        (DeviceWatcher::RemoveUpdated, watcher.Updated(&updated_handler(messages.clone()))?),
        (DeviceWatcher::RemoveRemoved, watcher.Removed(&removed_handler(messages.clone()))?),
        (
            DeviceWatcher::RemoveEnumerationCompleted,
            watcher.EnumerationCompleted(&enumeration_handler(messages.clone()))?,
        ),
    ];
    watcher.Start()?;
    Ok(WatcherSubscription { watcher, tokens })
}

fn added_handler(
    kind: LinkKind,
    messages: Sender<DeviceMessage>,
) -> TypedEventHandler<DeviceWatcher, DeviceInformation> {
    TypedEventHandler::new(move |_, information| {
        let information = information.ok()?;
        let message = DeviceMessage::Added {
            id: information.Id()?.to_string(),
            kind,
            name: information.Name()?.to_string(),
            patch: PropertyPatch::from_properties(&information.Properties()?),
        };
        let _ = messages.send(message);
        Ok(())
    })
}

fn updated_handler(messages: Sender<DeviceMessage>) -> TypedEventHandler<DeviceWatcher, DeviceInformationUpdate> {
    TypedEventHandler::new(move |_, update| {
        let update = update.ok()?;
        let message = DeviceMessage::Updated {
            id: update.Id()?.to_string(),
            patch: PropertyPatch::from_properties(&update.Properties()?),
        };
        let _ = messages.send(message);
        Ok(())
    })
}

fn removed_handler(messages: Sender<DeviceMessage>) -> TypedEventHandler<DeviceWatcher, DeviceInformationUpdate> {
    TypedEventHandler::new(move |_, update| {
        let _ = messages.send(DeviceMessage::Removed { id: update.ok()?.Id()?.to_string() });
        Ok(())
    })
}

fn enumeration_handler(
    messages: Sender<DeviceMessage>,
) -> TypedEventHandler<DeviceWatcher, windows_core::IInspectable> {
    TypedEventHandler::new(move |_, _| {
        let _ = messages.send(DeviceMessage::EnumerationCompleted);
        Ok(())
    })
}
