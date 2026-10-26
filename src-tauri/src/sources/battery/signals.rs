use crossbeam_channel::{bounded, Receiver, Sender};
use windows::Foundation::EventHandler;
use windows::System::Power::PowerManager;
use windows_core::{IInspectable, Result};

type Subscribe = fn(&EventHandler<IInspectable>) -> Result<i64>;
type Unsubscribe = fn(i64) -> Result<()>;

struct Registration {
    token: i64,
    unsubscribe: Unsubscribe,
}

pub struct PowerSignals {
    changes: Receiver<()>,
    registrations: Vec<Registration>,
}

impl PowerSignals {
    pub fn subscribe() -> Result<Self> {
        let (sender, changes) = bounded(1);
        let registrations = vec![
            register(
                &sender,
                |handler| PowerManager::BatteryStatusChanged(handler),
                PowerManager::RemoveBatteryStatusChanged,
            )?,
            register(
                &sender,
                |handler| PowerManager::PowerSupplyStatusChanged(handler),
                PowerManager::RemovePowerSupplyStatusChanged,
            )?,
            register(
                &sender,
                |handler| PowerManager::RemainingChargePercentChanged(handler),
                PowerManager::RemoveRemainingChargePercentChanged,
            )?,
        ];
        Ok(Self { changes, registrations })
    }

    pub fn take_pending_change(&self) -> bool {
        self.changes.try_iter().count() > 0
    }
}

impl Drop for PowerSignals {
    fn drop(&mut self) {
        for registration in &self.registrations {
            let _ = (registration.unsubscribe)(registration.token);
        }
    }
}

fn register(sender: &Sender<()>, subscribe: Subscribe, unsubscribe: Unsubscribe) -> Result<Registration> {
    let sender = sender.clone();
    let handler = EventHandler::<IInspectable>::new(move |_, _| {
        let _ = sender.try_send(());
        Ok(())
    });
    Ok(Registration { token: subscribe(&handler)?, unsubscribe })
}
