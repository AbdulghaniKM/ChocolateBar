use std::collections::HashMap;
use std::time::Instant;

use windows::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GetIfTable2, IF_TYPE_SOFTWARE_LOOPBACK, IF_TYPE_TUNNEL, MIB_IF_ROW2, MIB_IF_TABLE2,
};
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;

const HARDWARE_INTERFACE_BIT: u8 = 0b0000_0001;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Throughput {
    pub download_bytes_per_second: f64,
    pub upload_bytes_per_second: f64,
}

#[derive(Debug, Clone, Copy)]
struct OctetCounters {
    received: u64,
    sent: u64,
}

#[derive(Default)]
pub struct ThroughputMeter {
    previous: HashMap<u64, OctetCounters>,
    previous_at: Option<Instant>,
}

impl ThroughputMeter {
    pub fn sample(&mut self) -> Option<Throughput> {
        let counters = read_interface_counters()?;
        let now = Instant::now();
        let throughput = self.previous_at.map(|before| {
            let seconds = now.duration_since(before).as_secs_f64().max(f64::EPSILON);
            throughput_between(&self.previous, &counters, seconds)
        });
        self.previous = counters;
        self.previous_at = Some(now);
        throughput
    }
}

fn throughput_between(
    before: &HashMap<u64, OctetCounters>,
    after: &HashMap<u64, OctetCounters>,
    seconds: f64,
) -> Throughput {
    let (received, sent) = after
        .iter()
        .filter_map(|(luid, now)| before.get(luid).map(|earlier| (now, earlier)))
        .fold((0u64, 0u64), |(received, sent), (now, earlier)| {
            (
                received + now.received.saturating_sub(earlier.received),
                sent + now.sent.saturating_sub(earlier.sent),
            )
        });
    Throughput {
        download_bytes_per_second: received as f64 / seconds,
        upload_bytes_per_second: sent as f64 / seconds,
    }
}

fn read_interface_counters() -> Option<HashMap<u64, OctetCounters>> {
    let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
    if unsafe { GetIfTable2(&mut table) }.is_err() || table.is_null() {
        return None;
    }
    let counters = unsafe { collect_counters(table) };
    unsafe { FreeMibTable(table as *const _) };
    Some(counters)
}

unsafe fn collect_counters(table: *const MIB_IF_TABLE2) -> HashMap<u64, OctetCounters> {
    let rows = std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize);
    rows.iter()
        .filter(|row| counts_toward_throughput(row))
        .map(|row| {
            let luid = row.InterfaceLuid.Value;
            (luid, OctetCounters { received: row.InOctets, sent: row.OutOctets })
        })
        .collect()
}

fn counts_toward_throughput(row: &MIB_IF_ROW2) -> bool {
    let is_hardware = row.InterfaceAndOperStatusFlags._bitfield & HARDWARE_INTERFACE_BIT != 0;
    row.OperStatus == IfOperStatusUp
        && row.Type != IF_TYPE_SOFTWARE_LOOPBACK
        && row.Type != IF_TYPE_TUNNEL
        && is_hardware
}
