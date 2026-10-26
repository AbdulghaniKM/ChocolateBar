use std::path::PathBuf;

use crossbeam_channel::Sender;
use tauri::{AppHandle, DragDropEvent, Manager, WindowEvent};

const MAIN_WINDOW_LABEL: &str = "main";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropSignal {
    Hover(bool),
    Dropped(Vec<PathBuf>),
}

pub fn attach_to_main_window(app: &AppHandle, signals: Sender<DropSignal>) -> bool {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return false;
    };
    window.on_window_event(move |event| {
        if let WindowEvent::DragDrop(drag_event) = event {
            if let Some(signal) = signal_for(drag_event) {
                let _ = signals.send(signal);
            }
        }
    });
    true
}

fn signal_for(event: &DragDropEvent) -> Option<DropSignal> {
    match event {
        DragDropEvent::Enter { .. } => Some(DropSignal::Hover(true)),
        DragDropEvent::Leave => Some(DropSignal::Hover(false)),
        DragDropEvent::Drop { paths, .. } => Some(DropSignal::Dropped(paths.clone())),
        _ => None,
    }
}
