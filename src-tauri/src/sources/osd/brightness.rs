use std::thread;
use std::time::Duration;

use crossbeam_channel::Sender;
use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use wmi::{WMIConnection, WMIError, WMIResult};

const WMI_NAMESPACE: &str = "ROOT\\WMI";
const BRIGHTNESS_EVENT_QUERY: &str = "SELECT * FROM WmiMonitorBrightnessEvent";
const SET_BRIGHTNESS_METHOD: &str = "WmiSetBrightness";
const WATCH_RETRY_DELAY: Duration = Duration::from_secs(120);

#[derive(Deserialize)]
struct WmiMonitorBrightnessEvent {
    #[serde(rename = "Brightness")]
    brightness: u8,
}

#[derive(Deserialize)]
struct WmiMonitorBrightnessMethods {
    #[serde(rename = "__Path")]
    path: String,
}

#[derive(Serialize)]
struct SetBrightnessArguments {
    #[serde(rename = "Timeout")]
    timeout: u32,
    #[serde(rename = "Brightness")]
    brightness: u8,
}

pub struct BrightnessControl {
    connection: WMIConnection,
}

impl BrightnessControl {
    pub fn connect() -> WMIResult<Self> {
        Ok(Self { connection: WMIConnection::with_namespace_path(WMI_NAMESPACE)? })
    }

    pub fn set_percent(&self, percent: u8) -> WMIResult<()> {
        let arguments = SetBrightnessArguments { timeout: 0, brightness: percent.min(100) };
        for target in self.method_targets()? {
            let _: IgnoredAny = self.connection.exec_instance_method::<WmiMonitorBrightnessMethods, _>(
                &target.path,
                SET_BRIGHTNESS_METHOD,
                &arguments,
            )?;
        }
        Ok(())
    }

    fn method_targets(&self) -> WMIResult<Vec<WmiMonitorBrightnessMethods>> {
        self.connection.raw_query("SELECT * FROM WmiMonitorBrightnessMethods WHERE Active = TRUE")
    }
}

enum WatchStop {
    ReceiverGone,
    Failed(WMIError),
}

impl From<WMIError> for WatchStop {
    fn from(error: WMIError) -> Self {
        Self::Failed(error)
    }
}

pub fn spawn_brightness_watcher(sender: Sender<u8>) {
    let spawned = thread::Builder::new()
        .name("osd-brightness-events".into())
        .spawn(move || keep_watching(&sender));
    if let Err(error) = spawned {
        log::warn!("brightness watcher failed to start: {error}");
    }
}

fn keep_watching(sender: &Sender<u8>) {
    loop {
        match watch_events(sender) {
            Err(WatchStop::ReceiverGone) => return,
            Err(WatchStop::Failed(error)) => log::debug!("brightness events unavailable: {error}"),
            Ok(()) => {}
        }
        thread::sleep(WATCH_RETRY_DELAY);
    }
}

fn watch_events(sender: &Sender<u8>) -> Result<(), WatchStop> {
    let connection = WMIConnection::with_namespace_path(WMI_NAMESPACE)?;
    for event in connection.raw_notification::<WmiMonitorBrightnessEvent>(BRIGHTNESS_EVENT_QUERY)? {
        let brightness = event?.brightness;
        sender.send(brightness).map_err(|_| WatchStop::ReceiverGone)?;
    }
    Ok(())
}
