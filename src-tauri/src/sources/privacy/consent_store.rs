use std::path::Path;

use super::registry::RegistryKey;

const CONSENT_STORE_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore";
const NON_PACKAGED_KEY: &str = "NonPackaged";
const LAST_USED_START: &str = "LastUsedTimeStart";
const LAST_USED_STOP: &str = "LastUsedTimeStop";

#[derive(Clone, Copy)]
pub enum Capability {
    Microphone,
    Camera,
}

impl Capability {
    fn registry_name(self) -> &'static str {
        match self {
            Self::Microphone => "microphone",
            Self::Camera => "webcam",
        }
    }
}

pub fn apps_in_use(capability: Capability) -> Vec<String> {
    let path = format!("{CONSENT_STORE_PATH}\\{}", capability.registry_name());
    let Some(root) = RegistryKey::open_current_user(&path) else { return Vec::new() };
    let mut apps = in_use_children(&root, package_display_name);
    if let Some(non_packaged) = root.open_subkey(NON_PACKAGED_KEY) {
        apps.extend(in_use_children(&non_packaged, executable_display_name));
    }
    apps.sort();
    apps.dedup();
    apps
}

fn in_use_children(parent: &RegistryKey, display_name: fn(&str) -> String) -> Vec<String> {
    parent
        .subkey_names()
        .into_iter()
        .filter(|name| name != NON_PACKAGED_KEY)
        .filter(|name| parent.open_subkey(name).is_some_and(|key| is_in_use(&key)))
        .map(|name| display_name(&name))
        .collect()
}

fn is_in_use(app_key: &RegistryKey) -> bool {
    let started = app_key.read_qword(LAST_USED_START).unwrap_or(0) > 0;
    started && app_key.read_qword(LAST_USED_STOP) == Some(0)
}

fn package_display_name(package_key: &str) -> String {
    let identity = package_key.split('_').next().unwrap_or(package_key);
    let product = identity.rsplit('.').next().unwrap_or(identity);
    split_camel_case(product)
}

fn executable_display_name(executable_key: &str) -> String {
    let path = executable_key.replace('#', "\\");
    let stem = Path::new(&path).file_stem().and_then(|stem| stem.to_str()).unwrap_or(&path);
    capitalize_first(stem)
}

fn split_camel_case(text: &str) -> String {
    let mut spaced = String::with_capacity(text.len() + 4);
    let mut previous: Option<char> = None;
    for character in text.chars() {
        if character.is_uppercase() && previous.is_some_and(char::is_lowercase) {
            spaced.push(' ');
        }
        spaced.push(character);
        previous = Some(character);
    }
    spaced
}

fn capitalize_first(text: &str) -> String {
    let mut characters = text.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_names_become_readable_product_names() {
        assert_eq!(package_display_name("Microsoft.WindowsCamera_8wekyb3d8bbwe"), "Windows Camera");
    }

    #[test]
    fn non_packaged_keys_become_capitalized_executable_names() {
        let key = "C:#Program Files#Google#Chrome#Application#chrome.exe";
        assert_eq!(executable_display_name(key), "Chrome");
    }
}
