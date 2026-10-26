use crate::core::activity::{ActionSpec, ActionStyle, Activity, CompactView, Priority};
use crate::core::source::SourceContext;

use super::view::{build_activity, TimerView};

const ALERT_MODE: &str = "alert";
const BEEP_EXCLAMATION: u32 = 0x30;

windows_core::link!("user32.dll" "system" fn MessageBeep(utype: u32) -> i32);

pub struct Alert {
    pub key: &'static str,
    pub icon: &'static str,
    pub title: String,
    pub message: String,
    pub accent: &'static str,
    pub ttl_ms: u64,
}

impl Alert {
    pub fn announce(&self, ctx: &SourceContext) {
        ctx.upsert(self.activity());
        play_alert_sound();
    }

    fn activity(&self) -> Activity {
        let compact = CompactView::new(self.icon, self.title.clone())
            .subtitle(self.message.clone())
            .accent(self.accent);
        let view = TimerView {
            mode: ALERT_MODE,
            label: self.message.clone(),
            accent: self.accent,
            ..TimerView::default()
        };
        let dismiss = ActionSpec::button("dismiss", "Dismiss").style(ActionStyle::Primary);
        build_activity(self.key, compact, &view, vec![dismiss])
            .priority(Priority::Critical)
            .alert(true)
            .ttl_ms(self.ttl_ms)
    }
}

fn play_alert_sound() {
    unsafe {
        MessageBeep(BEEP_EXCLAMATION);
    }
}
