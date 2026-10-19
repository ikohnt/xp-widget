//! Application-wide events. Background threads (IPC, file watcher, image
//! decoding, file dialogs) and the tray window all talk to the UI thread by
//! sending an `AppEvent` through the winit event loop proxy.


use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use winit::event_loop::EventLoopProxy;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetCommand {
    ChangeImage,
    ChooseFolder,
    Next,
    Prev,
    Settings,
    Hide,
}

#[allow(dead_code)]
pub enum AppEvent {
    /// `--reload` or tray "Reload": re-read config.toml and reload every image.
    Reload,
    /// `--quit` or a "Quit" menu item.
    Quit,
    /// config.toml changed on disk.
    ConfigFileChanged,
    SaveConfig,
    OpenSettings,
    ShowAllWidgets,
    AddWidget,
    EditConfigFile,
    Widget {
        uid: u64,
        command: WidgetCommand,
    },
    ImagePicked {
        uid: u64,
        path: PathBuf,
    },
    ImageDecoded {
        uid: u64,
        generation: u64,
        result: Result<crate::image_src::LoadedImage, String>,
    },
    TrayMenuClick {
        id: String,
    },
    AccessKit(accesskit_winit::Event),
}

impl From<accesskit_winit::Event> for AppEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        AppEvent::AccessKit(event)
    }
}


static PROXY: OnceLock<Mutex<EventLoopProxy<AppEvent>>> = OnceLock::new();

pub fn init(proxy: EventLoopProxy<AppEvent>) {
    let _ = PROXY.set(Mutex::new(proxy));
}

pub fn get_proxy() -> Option<EventLoopProxy<AppEvent>> {
    PROXY.get().and_then(|p| p.lock().ok().map(|p| p.clone()))
}

/// Sends an event to the UI thread. Safe to call from any thread.
pub fn send(event: AppEvent) {
    if let Some(proxy) = PROXY.get() {
        if let Ok(proxy) = proxy.lock() {
            let _ = proxy.send_event(event);
        }
    }
}
