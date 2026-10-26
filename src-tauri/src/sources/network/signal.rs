use crossbeam_channel::{bounded, Receiver};
use windows::Networking::Connectivity::{NetworkInformation, NetworkStatusChangedEventHandler};
use windows_core::Result;

pub struct NetworkChangeSignal {
    changes: Receiver<()>,
    token: i64,
}

impl NetworkChangeSignal {
    pub fn subscribe() -> Result<Self> {
        let (sender, changes) = bounded(1);
        let handler = NetworkStatusChangedEventHandler::new(move |_| {
            let _ = sender.try_send(());
            Ok(())
        });
        let token = NetworkInformation::NetworkStatusChanged(&handler)?;
        Ok(Self { changes, token })
    }

    pub fn take_pending_change(&self) -> bool {
        self.changes.try_iter().count() > 0
    }
}

impl Drop for NetworkChangeSignal {
    fn drop(&mut self) {
        let _ = NetworkInformation::RemoveNetworkStatusChanged(self.token);
    }
}
