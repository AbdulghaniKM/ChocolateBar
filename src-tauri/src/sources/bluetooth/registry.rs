use std::collections::HashMap;

use super::properties::{normalize_address, LinkKind, PropertyPatch};
use super::watcher::DeviceMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceSummary {
    pub name: String,
    pub connected: bool,
    pub battery: Option<u8>,
}

pub struct DeviceDelta {
    pub key: String,
    pub before: Option<DeviceSummary>,
    pub after: Option<DeviceSummary>,
}

impl DeviceDelta {
    pub fn has_changed(&self) -> bool {
        self.before != self.after
    }
}

struct LinkState {
    kind: LinkKind,
    connected: bool,
    battery: Option<u8>,
}

struct DeviceRecord {
    name: String,
    links: HashMap<String, LinkState>,
}

impl DeviceRecord {
    fn summary(&self) -> DeviceSummary {
        let connected = self.links.values().any(|link| link.connected);
        DeviceSummary { name: self.name.clone(), connected, battery: self.battery_if(connected) }
    }

    fn battery_if(&self, connected: bool) -> Option<u8> {
        if !connected {
            return None;
        }
        let connected_links = || self.links.values().filter(|link| link.connected);
        connected_links()
            .filter(|link| link.kind == LinkKind::Classic)
            .find_map(|link| link.battery)
            .or_else(|| connected_links().find_map(|link| link.battery))
    }
}

#[derive(Default)]
pub struct DeviceRegistry {
    records: HashMap<String, DeviceRecord>,
    key_by_link_id: HashMap<String, String>,
}

impl DeviceRegistry {
    pub fn apply(&mut self, message: DeviceMessage) -> Option<DeviceDelta> {
        match message {
            DeviceMessage::Added { id, kind, name, patch } => self.add_link(id, kind, name, patch),
            DeviceMessage::Updated { id, patch } => self.update_link(&id, patch),
            DeviceMessage::Removed { id } => self.remove_link(&id),
            DeviceMessage::EnumerationCompleted => None,
        }
    }

    pub fn low_energy_links(&self, key: &str) -> Vec<(String, bool)> {
        let Some(record) = self.records.get(key) else {
            return Vec::new();
        };
        record
            .links
            .iter()
            .filter(|(_, link)| link.kind == LinkKind::LowEnergy)
            .map(|(id, link)| (id.clone(), link.connected))
            .collect()
    }

    fn add_link(&mut self, id: String, kind: LinkKind, name: String, patch: PropertyPatch) -> Option<DeviceDelta> {
        if name.trim().is_empty() {
            return None;
        }
        let key = patch.address.clone().unwrap_or_else(|| normalize_address(&id));
        let before = self.records.get(&key).map(DeviceRecord::summary);
        let record = self
            .records
            .entry(key.clone())
            .or_insert_with(|| DeviceRecord { name: name.clone(), links: HashMap::new() });
        record.links.insert(
            id.clone(),
            LinkState { kind, connected: patch.connected.unwrap_or(false), battery: patch.battery },
        );
        self.key_by_link_id.insert(id, key.clone());
        self.delta_for(key, before)
    }

    fn update_link(&mut self, id: &str, patch: PropertyPatch) -> Option<DeviceDelta> {
        let key = self.key_by_link_id.get(id)?.clone();
        let record = self.records.get_mut(&key)?;
        let before = Some(record.summary());
        let link = record.links.get_mut(id)?;
        if let Some(connected) = patch.connected {
            link.connected = connected;
        }
        if patch.battery.is_some() {
            link.battery = patch.battery;
        }
        self.delta_for(key, before)
    }

    fn remove_link(&mut self, id: &str) -> Option<DeviceDelta> {
        let key = self.key_by_link_id.remove(id)?;
        let record = self.records.get_mut(&key)?;
        let before = Some(record.summary());
        record.links.remove(id);
        if record.links.is_empty() {
            self.records.remove(&key);
        }
        self.delta_for(key, before)
    }

    fn delta_for(&self, key: String, before: Option<DeviceSummary>) -> Option<DeviceDelta> {
        let after = self.records.get(&key).map(DeviceRecord::summary);
        Some(DeviceDelta { key, before, after })
    }
}
