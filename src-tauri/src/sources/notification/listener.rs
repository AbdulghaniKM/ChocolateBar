use std::collections::HashSet;

use windows::UI::Notifications::Management::{UserNotificationListener, UserNotificationListenerAccessStatus};
use windows::UI::Notifications::{NotificationKinds, UserNotification};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub id: u32,
    pub title: String,
    pub body: String,
}

pub struct ToastWatcher {
    listener: UserNotificationListener,
    seen_ids: Option<HashSet<u32>>,
}

impl ToastWatcher {
    pub fn request_access() -> windows::core::Result<Option<Self>> {
        let listener = UserNotificationListener::Current()?;
        let status = listener.RequestAccessAsync()?.join()?;
        let allowed = status == UserNotificationListenerAccessStatus::Allowed;
        Ok(allowed.then_some(Self { listener, seen_ids: None }))
    }

    pub fn poll_new_toasts(&mut self) -> windows::core::Result<Vec<Toast>> {
        let current = self.current_toasts()?;
        let current_ids: HashSet<u32> = current.iter().map(|toast| toast.id).collect();
        let previously_seen = self.seen_ids.replace(current_ids);
        let Some(previously_seen) = previously_seen else {
            return Ok(Vec::new());
        };
        Ok(current.into_iter().filter(|toast| !previously_seen.contains(&toast.id)).collect())
    }

    fn current_toasts(&self) -> windows::core::Result<Vec<Toast>> {
        let notifications = self.listener.GetNotificationsAsync(NotificationKinds::Toast)?.join()?;
        notifications.into_iter().map(|notification| read_toast(&notification)).collect()
    }
}

fn read_toast(notification: &UserNotification) -> windows::core::Result<Toast> {
    let mut lines = text_lines_of(notification)?.into_iter();
    let title = lines.next().unwrap_or_default();
    let body = lines.collect::<Vec<_>>().join("\n");
    Ok(Toast { id: notification.Id()?, title, body })
}

fn text_lines_of(notification: &UserNotification) -> windows::core::Result<Vec<String>> {
    let bindings = notification.Notification()?.Visual()?.Bindings()?;
    let mut lines = Vec::new();
    for binding in bindings {
        for element in binding.GetTextElements()? {
            lines.push(element.Text()?.to_string());
        }
    }
    Ok(lines)
}
