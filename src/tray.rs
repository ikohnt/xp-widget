use muda::{Menu, MenuItem, CheckMenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::events::{self, AppEvent};

#[allow(dead_code)]
pub struct TraySetup {
    pub tray_icon: TrayIcon,
    pub menu: Menu,
    pub reload_id: String,
    pub quit_id: String,
    pub edit_config_id: String,
    pub settings_id: String,
    pub add_widget_id: String,
    pub show_all_id: String,
    pub startup_id: String,
    pub startup_item: CheckMenuItem,
}

pub fn get_auto_launch() -> auto_launch::AutoLaunch {
    let app_path = std::env::current_exe().unwrap().to_string_lossy().to_string();
    auto_launch::AutoLaunchBuilder::new()
        .set_app_name("xp-widget")
        .set_app_path(&app_path)
        .build()
        .unwrap()
}

pub fn create_tray() -> Option<TraySetup> {
    let menu = Menu::new();
    let reload_item = MenuItem::new("Reload Configuration", true, None);
    let edit_config_item = MenuItem::new("Edit Config (config.toml)", true, None);
    let settings_item = MenuItem::new("Settings", true, None);
    let add_widget_item = MenuItem::new("Add Widget", true, None);
    let show_all_item = MenuItem::new("Show all widgets", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    
    let al = get_auto_launch();
    let is_enabled = al.is_enabled().unwrap_or(false);
    let startup_item = CheckMenuItem::new("Start with Windows", true, is_enabled, None);

    let reload_id = reload_item.id().0.clone();
    let edit_config_id = edit_config_item.id().0.clone();
    let settings_id = settings_item.id().0.clone();
    let add_widget_id = add_widget_item.id().0.clone();
    let show_all_id = show_all_item.id().0.clone();
    let startup_id = startup_item.id().0.clone();
    let quit_id = quit_item.id().0.clone();

    let _ = menu.append(&add_widget_item);
    let _ = menu.append(&show_all_item);
    let _ = menu.append(&startup_item);
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&settings_item);
    let _ = menu.append(&reload_item);
    let _ = menu.append(&edit_config_item);
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&quit_item);

    // Load embedded tray icon
    let icon_data = include_bytes!("../assets/icon.jpg");
    let icon = if let Ok(img) = image::load_from_memory(icon_data) {
        let img = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3).to_rgba8();
        let (width, height) = img.dimensions();
        let rgba = img.into_raw();
        Icon::from_rgba(rgba, width, height).ok()
    } else {
        None
    };

    // Fallback to 16x16 dummy tray icon (solid navy blue) if embedded fails
    let icon = icon.or_else(|| {
        let mut icon_rgba = vec![0; 16 * 16 * 4];
        for chunk in icon_rgba.chunks_mut(4) {
            chunk[0] = 0; // R
            chunk[1] = 0; // G
            chunk[2] = 170; // B
            chunk[3] = 255; // A
        }
        Icon::from_rgba(icon_rgba, 16, 16).ok()
    })?;

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu.clone()))
        .with_tooltip("XP Widget")
        .with_icon(icon)
        .build()
        .ok()?;

    Some(TraySetup {
        tray_icon,
        menu,
        reload_id,
        quit_id,
        edit_config_id,
        settings_id,
        add_widget_id,
        show_all_id,
        startup_id,
        startup_item,
    })
}

pub fn spawn_event_pump() {
    std::thread::spawn(|| {
        let menu_rx = muda::MenuEvent::receiver();
        loop {
            if let Ok(event) = menu_rx.try_recv() {
                events::send(AppEvent::TrayMenuClick { id: event.id.0 });
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });
}
