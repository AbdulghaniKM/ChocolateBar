use windows::Win32::Foundation::ERROR_BUFFER_OVERFLOW;
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_SKIP_MULTICAST,
    GAA_FLAG_SKIP_UNICAST, IF_TYPE_PPP, IF_TYPE_PROP_VIRTUAL, IF_TYPE_TUNNEL, IP_ADAPTER_ADDRESSES_LH,
};
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
use windows::Win32::Networking::WinSock::AF_UNSPEC;

const MAX_BUFFER_ATTEMPTS: usize = 3;
const INITIAL_BUFFER_BYTES: u32 = 16 * 1024;

const VPN_KEYWORDS: [&str; 16] = [
    "wireguard",
    "openvpn",
    "tap-windows",
    "tap-win",
    "tailscale",
    "nordlynx",
    "wintun",
    "zerotier",
    "anyconnect",
    "forticlient",
    "globalprotect",
    "proton",
    "mullvad",
    "warp",
    "pangp",
    "vpn",
];

const NON_VPN_KEYWORDS: [&str; 9] = [
    "hyper-v",
    "vmware",
    "virtualbox",
    "teredo",
    "6to4",
    "isatap",
    "ip-https",
    "bluetooth",
    "wi-fi direct",
];

pub fn is_vpn_active() -> bool {
    let Some(buffer) = read_adapter_buffer() else {
        return false;
    };
    let mut current = buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
    while !current.is_null() {
        let adapter = unsafe { &*current };
        if adapter.OperStatus == IfOperStatusUp && is_vpn_adapter(adapter.IfType, &description_of(adapter)) {
            return true;
        }
        current = adapter.Next;
    }
    false
}

fn read_adapter_buffer() -> Option<Vec<u64>> {
    let flags = GAA_FLAG_SKIP_UNICAST | GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;
    let mut size = INITIAL_BUFFER_BYTES;
    for _ in 0..MAX_BUFFER_ATTEMPTS {
        let mut buffer = vec![0u64; (size as usize).div_ceil(size_of::<u64>())];
        let result = unsafe {
            GetAdaptersAddresses(
                u32::from(AF_UNSPEC.0),
                flags,
                None,
                Some(buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
                &mut size,
            )
        };
        match result {
            0 => return Some(buffer),
            code if code == ERROR_BUFFER_OVERFLOW.0 => continue,
            _ => return None,
        }
    }
    None
}

fn description_of(adapter: &IP_ADAPTER_ADDRESSES_LH) -> String {
    let description = unsafe { adapter.Description.to_string() }.unwrap_or_default();
    let friendly_name = unsafe { adapter.FriendlyName.to_string() }.unwrap_or_default();
    format!("{description} {friendly_name}").to_lowercase()
}

fn is_vpn_adapter(interface_type: u32, lowercase_description: &str) -> bool {
    if NON_VPN_KEYWORDS.iter().any(|keyword| lowercase_description.contains(keyword)) {
        return false;
    }
    let is_tunnel_like = matches!(interface_type, IF_TYPE_PPP | IF_TYPE_TUNNEL | IF_TYPE_PROP_VIRTUAL);
    is_tunnel_like || VPN_KEYWORDS.iter().any(|keyword| lowercase_description.contains(keyword))
}
