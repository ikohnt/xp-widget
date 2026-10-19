use std::collections::HashMap;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Window, WindowId, ResizeDirection, CursorIcon};
use winit::platform::windows::WindowAttributesExtWindows;

use crate::config::{self, AppConfig, Layer, WidgetConfig};
use crate::events::{self, AppEvent, WidgetCommand};
use crate::layer;
use crate::log;
use windows_sys::Win32::Graphics::Dwm::{DwmEnableBlurBehindWindow, DWM_BLURBEHIND, DWM_BB_ENABLE};
use windows_sys::Win32::Foundation::HWND;
use crate::render::WidgetLayout;
use crate::tray;
use crate::widget::WidgetState;

pub struct App {
    pub app_config: AppConfig,
    pub widgets: HashMap<WindowId, WidgetState>,
    pub next_uid: u64,
    pub tray_setup: Option<tray::TraySetup>,

    // Context menu
    pub context_menu: muda::Menu,
    pub edit_menu: muda::Menu,
    pub view_menu: muda::Menu,
    pub help_menu: muda::Menu,
    pub change_img_id: String,
    pub choose_folder_id: String,
    pub rotate_id: String,
    pub toggle_lock_id: String,
    pub cycle_fit_id: String,
    pub duplicate_id: String,
    pub remove_id: String,
    pub reload_config_id: String,
    pub theme_classic_id: String,
    pub theme_98_id: String,
    pub theme_aero_id: String,
    pub theme_aqua_id: String,
    pub scale_50_id: String,
    pub scale_100_id: String,
    pub scale_150_id: String,
    pub scale_200_id: String,
    pub active_widget_uid: Option<u64>,
}

impl App {
    pub fn new(app_config: AppConfig) -> Self {
        let context_menu = muda::Menu::new();
        let change_img_item = muda::MenuItem::new("Change Image...", true, None);
        let choose_folder_item = muda::MenuItem::new("Choose Folder...", true, None);
        let rotate_item = muda::MenuItem::new("Rotate 90°", true, None);
        let cycle_fit_item = muda::MenuItem::new("Cycle Fit/Stretch", true, None);
        let toggle_lock_item = muda::MenuItem::new("Toggle Lock Position", true, None);
        
        let theme_menu = muda::Submenu::new("Theme", true);
        let theme_classic_item = muda::MenuItem::new("Classic (Light Gray)", true, None);
        let theme_98_item = muda::MenuItem::new("Win98 (Gradient)", true, None);
        let theme_aero_item = muda::MenuItem::new("Aero Glass", true, None);
        let theme_aqua_item = muda::MenuItem::new("MacOS Aqua", true, None);
        let _ = theme_menu.append(&theme_classic_item);
        let _ = theme_menu.append(&theme_98_item);
        let _ = theme_menu.append(&theme_aero_item);
        let _ = theme_menu.append(&theme_aqua_item);

        let scale_menu = muda::Submenu::new("Scale", true);
        let scale_50_item = muda::MenuItem::new("50%", true, None);
        let scale_100_item = muda::MenuItem::new("100%", true, None);
        let scale_150_item = muda::MenuItem::new("150%", true, None);
        let scale_200_item = muda::MenuItem::new("200%", true, None);
        let _ = scale_menu.append(&scale_50_item);
        let _ = scale_menu.append(&scale_100_item);
        let _ = scale_menu.append(&scale_150_item);
        let _ = scale_menu.append(&scale_200_item);

        let duplicate_item = muda::MenuItem::new("Duplicate Widget", true, None);
        let remove_item = muda::MenuItem::new("Remove Widget", true, None);
        let reload_config_item = muda::MenuItem::new("Reload Config", true, None);
        
        let _ = context_menu.append(&change_img_item);
        let _ = context_menu.append(&choose_folder_item);
        let _ = context_menu.append(&muda::PredefinedMenuItem::separator());
        let _ = context_menu.append(&rotate_item);
        let _ = context_menu.append(&cycle_fit_item);
        let _ = context_menu.append(&toggle_lock_item);
        let _ = context_menu.append(&muda::PredefinedMenuItem::separator());
        let _ = context_menu.append(&theme_menu);
        let _ = context_menu.append(&scale_menu);
        let _ = context_menu.append(&muda::PredefinedMenuItem::separator());
        let _ = context_menu.append(&duplicate_item);
        let _ = context_menu.append(&remove_item);
        let _ = context_menu.append(&muda::PredefinedMenuItem::separator());
        let _ = context_menu.append(&reload_config_item);

        let edit_menu = muda::Menu::new();
        let view_menu = muda::Menu::new();
        let help_menu = muda::Menu::new();

        let _ = edit_menu.append(&change_img_item);
        let _ = edit_menu.append(&choose_folder_item);
        let _ = edit_menu.append(&muda::PredefinedMenuItem::separator());
        let _ = edit_menu.append(&duplicate_item);
        let _ = edit_menu.append(&remove_item);

        let _ = view_menu.append(&rotate_item);
        let _ = view_menu.append(&cycle_fit_item);
        let _ = view_menu.append(&toggle_lock_item);

        let _ = help_menu.append(&reload_config_item);

        let tray_setup = tray::create_tray();
        if tray_setup.is_some() {
            tray::spawn_event_pump();
        }

        Self {
            app_config,
            widgets: HashMap::new(),
            next_uid: 0,
            tray_setup,
            context_menu,
            edit_menu,
            view_menu,
            help_menu,
            change_img_id: change_img_item.id().0.clone(),
            choose_folder_id: choose_folder_item.id().0.clone(),
            rotate_id: rotate_item.id().0.clone(),
            cycle_fit_id: cycle_fit_item.id().0.clone(),
            toggle_lock_id: toggle_lock_item.id().0.clone(),
            duplicate_id: duplicate_item.id().0.clone(),
            remove_id: remove_item.id().0.clone(),
            reload_config_id: reload_config_item.id().0.clone(),
            theme_classic_id: theme_classic_item.id().0.clone(),
            theme_98_id: theme_98_item.id().0.clone(),
            theme_aero_id: theme_aero_item.id().0.clone(),
            theme_aqua_id: theme_aqua_item.id().0.clone(),
            scale_50_id: scale_50_item.id().0.clone(),
            scale_100_id: scale_100_item.id().0.clone(),
            scale_150_id: scale_150_item.id().0.clone(),
            scale_200_id: scale_200_item.id().0.clone(),
            active_widget_uid: None,
        }
    }

    fn reload_config(&mut self, event_loop: &ActiveEventLoop) {
        let loaded = config::load_or_create();
        if let Some(err) = loaded.error {
            log::error(&format!("Skipping reload due to config error: {err}"));
            return;
        }
        self.app_config = loaded.config;
        log::verbose("Configuration reloaded");
        let old_len = self.widgets.len();
        let new_len = self.app_config.widgets.len();

        let mut existing_windows: Vec<winit::window::WindowId> = self.widgets.keys().copied().collect();
        // Sort by config_index so we match correctly
        existing_windows.sort_by_key(|id| self.widgets[id].config_index);

        for (i, w_config) in self.app_config.widgets.clone().into_iter().enumerate() {
            if i < old_len {
                // Update existing
                let win_id = existing_windows[i];
                let widget = self.widgets.get_mut(&win_id).unwrap();
                let old_config = std::mem::replace(&mut widget.config, w_config);
                if widget.config.ghost_mode != old_config.ghost_mode && widget.config.layer != crate::config::Layer::Desktop {
                    if let Ok(handle) = widget.window.window_handle() {
                        if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                            crate::layer::set_ghost_mode(win32.hwnd.get(), widget.config.ghost_mode);
                        }
                    }
                }
                if widget.config != old_config {
                    widget.reload_image();
                }
            } else {
                // Create new
                self.create_widget_window(event_loop, w_config, i);
            }
        }

        // Remove extra
        if old_len > new_len {
            for win_id in &existing_windows[new_len..old_len] {
                self.widgets.remove(win_id);
            }
        }
    }

    fn save_config(&self) {
        let text = self.app_config.to_file_string();
        if let Some(cfg_path) = config::AppConfig::path() {
            let _ = std::fs::write(cfg_path, text);
        }
    }

    fn remove_widget(&mut self, window_id: WindowId, event_loop: &ActiveEventLoop) {
        if let Some(widget) = self.widgets.remove(&window_id) {
            let config_index = widget.config_index;
            if self.app_config.widgets.len() > config_index {
                self.app_config.widgets.remove(config_index);
                for w in self.widgets.values_mut() {
                    if w.config_index > config_index {
                        w.config_index -= 1;
                    }
                }
                self.save_config();
            }
        }
        if self.widgets.is_empty() {
            event_loop.exit();
        }
    }

    fn create_widget_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        config: WidgetConfig,
        index: usize,
    ) {
        let (logical_w, logical_h) = config.window_logical_size();

        let attrs = Window::default_attributes()
            .with_title(config.window_title.clone())
            .with_inner_size(LogicalSize::new(logical_w, logical_h))
            .with_position(winit::dpi::PhysicalPosition::new(config.x, config.y))
            .with_decorations(false)
            .with_transparent(true)
            .with_visible(false)
            .with_skip_taskbar(true);

        match event_loop.create_window(attrs) {
            Ok(window) => {
            let window = Rc::new(window);
            let context = softbuffer::Context::new(window.clone()).unwrap();
            let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

            if config.layer == Layer::Desktop {
                if let Ok(handle) = window.window_handle() {
                    if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                        layer::attach_to_desktop(win32_handle.hwnd.get());
                    }
                }
            } else if config.ghost_mode {
                if let Ok(handle) = window.window_handle() {
                    if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
                        layer::set_ghost_mode(win32_handle.hwnd.get(), true);
                    }
                }
            }
            
            if config.layer == Layer::Standard {
                if let Ok(handle) = window.window_handle() {
                    if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                        layer::enable_drop_shadow(win32.hwnd.get());
                    }
                }
            }

            clamp_to_monitor(&window);

            let uid = self.next_uid;
            self.next_uid += 1;

            let state = WidgetState::new(uid, index, config.clone(), window.clone(), surface, event_loop);
            self.widgets.insert(window.id(), state);
            
            if config.theme == "aero" {
                if let Ok(handle) = window.window_handle() {
                    if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                        let hwnd = win32.hwnd.get() as HWND;
                        let bb = DWM_BLURBEHIND {
                            dwFlags: DWM_BB_ENABLE,
                            fEnable: 1,
                            hRgnBlur: 0,
                            fTransitionOnMaximized: 0,
                        };
                        unsafe {
                            DwmEnableBlurBehindWindow(hwnd, &bb);
                        }
                    }
                }
            }
            
            window.set_visible(true);
        }
        Err(e) => {
            println!("Failed to create window: {:?}", e);
        }
    }
    }
}

pub fn clamp_to_monitor(window: &Window) {
    let pos = window
        .outer_position()
        .unwrap_or(winit::dpi::PhysicalPosition::new(0, 0));
    let size = window.outer_size();

    let mut monitors = Vec::new();
    for monitor in window.available_monitors() {
        let m_pos = monitor.position();
        let m_size = monitor.size();
        monitors.push((m_pos.x, m_pos.y, m_size.width, m_size.height));
    }

    if let Some((nx, ny)) = clamp_rect((pos.x, pos.y, size.width, size.height), &monitors) {
        log::info("Window off-screen, snapping to monitor");
        window.set_outer_position(winit::dpi::PhysicalPosition::new(nx, ny));
    }
}

pub fn clamp_rect(
    rect: (i32, i32, u32, u32),
    monitors: &[(i32, i32, u32, u32)],
) -> Option<(i32, i32)> {
    let mut visible = false;
    let primary = monitors.first()?;

    let (rx, ry, rw, rh) = rect;
    for &(mx, my, mw, mh) in monitors {
        if rx < mx + mw as i32 && rx + rw as i32 > mx && ry < my + mh as i32 && ry + rh as i32 > my
        {
            visible = true;
            break;
        }
    }

    if !visible {
        Some((primary.0 + 100, primary.1 + 100))
    } else {
        None
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::Reload | AppEvent::ConfigFileChanged => self.reload_config(event_loop),
            AppEvent::SaveConfig => self.save_config(),
            AppEvent::Quit => event_loop.exit(),
            AppEvent::OpenSettings => {
                log::info("OpenSettings requested");
                crate::settings_ui::open_settings();
            }
            AppEvent::ShowAllWidgets => {
                for widget in self.widgets.values_mut() {
                    widget.window.set_visible(true);
                }
            }
            AppEvent::AddWidget => {
                let mut new_widget = WidgetConfig::default();
                new_widget.x += (self.widgets.len() as i32 * 50) % 300;
                new_widget.y += (self.widgets.len() as i32 * 50) % 300;

                self.app_config.widgets.push(new_widget);
                self.save_config();
            }
            AppEvent::EditConfigFile => {
                if let Some(path) = config::AppConfig::path() {
                    let _ = std::process::Command::new("notepad").arg(path).spawn();
                }
            }
            AppEvent::TrayMenuClick { id } => {
                if let Some(tray) = &self.tray_setup {
                    if id == tray.reload_id {
                        self.reload_config(event_loop);
                    } else if id == tray.edit_config_id {
                        if let Some(path) = config::AppConfig::path() {
                            let _ = std::process::Command::new("notepad").arg(path).spawn();
                        }
                    } else if id == tray.add_widget_id {
                        events::send(AppEvent::AddWidget);
                    } else if id == tray.quit_id {
                        event_loop.exit();
                    } else if id == tray.settings_id {
                        events::send(AppEvent::OpenSettings);
                    } else if id == tray.show_all_id {
                        events::send(AppEvent::ShowAllWidgets);
                    } else if id == tray.startup_id {
                        let al = crate::tray::get_auto_launch();
                        let is_enabled = al.is_enabled().unwrap_or(false);
                        if is_enabled {
                            let _ = al.disable();
                            tray.startup_item.set_checked(false);
                        } else {
                            let _ = al.enable();
                            tray.startup_item.set_checked(true);
                        }
                    }
                }

                // Also handle context menu clicks
                if id == self.reload_config_id {
                    self.reload_config(event_loop);
                } else if let Some(uid) = self.active_widget_uid {
                    let mut config_index_opt = None;
                    if let Some(widget) = self.widgets.values().find(|w| w.uid == uid) {
                        config_index_opt = Some(widget.config_index);
                    }

                    if let Some(config_index) = config_index_opt {
                        let mut needs_reload = false;
                        if id == self.change_img_id {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "gif"])
                                .pick_file()
                            {
                                self.app_config.widgets[config_index].image_path = path.to_string_lossy().to_string();
                                self.save_config();
                            }
                        } else if id == self.choose_folder_id {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.app_config.widgets[config_index].image_path = path.to_string_lossy().to_string();
                                self.save_config();
                            }
                        } else if id == self.rotate_id {
                            let r = self.app_config.widgets[config_index].rotation;
                            self.app_config.widgets[config_index].rotation = (r + 90) % 360;
                            self.save_config();
                        } else if id == self.cycle_fit_id {
                            use crate::config::Fit;
                            let new_fit = match self.app_config.widgets[config_index].fit {
                                Fit::Fit => Fit::Stretch,
                                Fit::Stretch => Fit::Tile,
                                Fit::Tile => Fit::Contain,
                                Fit::Contain => Fit::Cover,
                                Fit::Cover => Fit::Fill,
                                Fit::Fill => Fit::None,
                                Fit::None => Fit::ScaleDown,
                                Fit::ScaleDown => Fit::Fit,
                            };
                            self.app_config.widgets[config_index].fit = new_fit;
                            self.save_config();
                        } else if id == self.toggle_lock_id {
                            self.app_config.widgets[config_index].locked = !self.app_config.widgets[config_index].locked;
                            self.save_config();
                        } else if id == self.duplicate_id {
                            let mut clone = self.app_config.widgets[config_index].clone();
                            clone.x += 50;
                            clone.y += 50;
                            self.app_config.widgets.push(clone);
                            self.save_config();
                        } else if id == self.remove_id {
                            // Find window_id for remove
                            let mut win_id = None;
                            if let Some(widget) = self.widgets.values().find(|w| w.uid == uid) {
                                win_id = Some(widget.window.id());
                            }
                            if let Some(wid) = win_id {
                                self.remove_widget(wid, event_loop);
                            }
                        } else if id == self.theme_classic_id {
                            self.app_config.widgets[config_index].theme = "classic".to_string();
                            self.save_config();
                        } else if id == self.theme_98_id {
                            self.app_config.widgets[config_index].theme = "98".to_string();
                            self.save_config();
                        } else if id == self.theme_aero_id {
                            self.app_config.widgets[config_index].theme = "aero".to_string();
                            self.save_config();
                        } else if id == self.theme_aqua_id {
                            self.app_config.widgets[config_index].theme = "aqua".to_string();
                            self.save_config();
                        } else if id == self.scale_50_id {
                            self.app_config.widgets[config_index].scale = 0.5;
                            self.save_config();
                            needs_reload = true;
                        } else if id == self.scale_100_id {
                            self.app_config.widgets[config_index].scale = 1.0;
                            self.save_config();
                            needs_reload = true;
                        } else if id == self.scale_150_id {
                            self.app_config.widgets[config_index].scale = 1.5;
                            self.save_config();
                            needs_reload = true;
                        } else if id == self.scale_200_id {
                            self.app_config.widgets[config_index].scale = 2.0;
                            self.save_config();
                            needs_reload = true;
                        }

                        if needs_reload {
                            if let Some(widget) = self.widgets.values_mut().find(|w| w.uid == uid) {
                                widget.reload_image();
                            }
                        }
                    }
                }
            }
            AppEvent::Widget { uid, command } => {
                for widget in self.widgets.values_mut() {
                    if widget.uid == uid {
                        match command {
                            WidgetCommand::Next => {
                                widget.source.next();
                                widget.begin_load();
                            }
                            WidgetCommand::Prev => {
                                widget.source.prev();
                                widget.begin_load();
                            }
                            _ => {}
                        }
                        break;
                    }
                }
            }
            AppEvent::ImagePicked { .. } => {}
            AppEvent::ImageDecoded { uid, generation, result } => {
                for widget in self.widgets.values_mut() {
                    if widget.uid == uid && widget.load_generation == generation {
                        let mut size_changed = false;
                        match result {
                            Ok(crate::image_src::LoadedImage::Static(img)) => {
                                let (img_w, img_h) = (img.width() as f32, img.height() as f32);
                                widget.img_state = Some(crate::image_src::ImageState::Static(img));
                                
                                if img_w > 0.0 && img_h > 0.0 {
                                    let ratio = img_w / img_h;
                                    widget.config.height = (widget.config.width as f32 / ratio).round() as u32;
                                    size_changed = true;
                                }
                            }
                            Ok(crate::image_src::LoadedImage::Animated { frames, delays }) => {
                                let (img_w, img_h) = (frames[0].width() as f32, frames[0].height() as f32);
                                widget.img_state = Some(crate::image_src::ImageState::Animated {
                                    frames,
                                    delays,
                                    current_frame: 0,
                                    last_frame_time: std::time::Instant::now(),
                                });
                                
                                if img_w > 0.0 && img_h > 0.0 {
                                    let ratio = img_w / img_h;
                                    widget.config.height = (widget.config.width as f32 / ratio).round() as u32;
                                    size_changed = true;
                                }
                            }
                            Err(e) => widget.img_state = Some(crate::image_src::ImageState::Error(e)),
                        }
                        
                        if size_changed {
                            self.app_config.widgets[widget.config_index].height = widget.config.height;
                            crate::events::send(crate::events::AppEvent::SaveConfig);
                            
                            let (logical_w, logical_h) = widget.config.window_logical_size();
                            let _ = widget.window.request_inner_size(winit::dpi::LogicalSize::new(logical_w, logical_h));
                        }

                        widget.update_accessibility_tree();
                        widget.window.request_redraw();
                        break;
                    }
                }
            }
            AppEvent::AccessKit(_) => {
                // Ignore incoming accesskit actions (we don't have interactive accessible elements)
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.widgets.is_empty() {
            let widgets_config = self.app_config.widgets.clone();
            for (i, w_config) in widgets_config.into_iter().enumerate() {
                self.create_widget_window(event_loop, w_config, i);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(widget) = self.widgets.get_mut(&window_id) else {
            return;
        };

        if let Some(adapter) = &mut widget.accesskit_adapter {
            adapter.process_event(&widget.window, &event);
        }

        match event {
            WindowEvent::CloseRequested => {
                self.remove_widget(window_id, event_loop);
            }
            WindowEvent::KeyboardInput {
                event: kb_event, ..
            } => {
                if kb_event.state == ElementState::Pressed {
                    match kb_event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => {
                            self.remove_widget(window_id, event_loop)
                        }
                        PhysicalKey::Code(KeyCode::ArrowLeft) => {
                            widget.source.prev();
                            widget.begin_load();
                        }
                        PhysicalKey::Code(KeyCode::ArrowRight) => {
                            widget.source.next();
                            widget.begin_load();
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if widget.config.layer == Layer::Desktop {
                    return;
                }

                if state == ElementState::Pressed {
                    if button == MouseButton::Left {
                        widget.left_mouse_pressed = true;
                        widget.window.request_redraw();
                    }
                    if button == MouseButton::Right && widget.config.layer == Layer::Standard {
                        self.active_widget_uid = Some(widget.uid);
                        if let Ok(handle) = widget.window.window_handle() {
                            if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                                use muda::ContextMenu;
                                unsafe {
                                    self.context_menu
                                        .show_context_menu_for_hwnd(win32.hwnd.get(), None);
                                }
                            }
                        }
                    } else if button == MouseButton::Left {
                        if widget.config.shows_chrome() {
                            let size = widget.window.inner_size();
                            let scale_factor = widget.window.scale_factor() as f32;
                            let layout = WidgetLayout::new(
                                &widget.config,
                                size.width as f32,
                                size.height as f32,
                                scale_factor,
                            );

                            let (cx, cy, cw, ch) = layout.close_button_rect();
                            let (mx_x, mn_x) = if layout.is_aqua {
                                let space = 4.0 * layout.scale;
                                (cx + 2.0 * (cw + space), cx + cw + space)
                            } else {
                                (cx - cw - 2.0 * layout.scale, cx - 2.0 * cw - 4.0 * layout.scale)
                            };

                            let mx = widget.cursor_pos.0;
                            let my = widget.cursor_pos.1;

                            if mx >= cx && mx <= cx + cw && my >= cy && my <= cy + ch {
                                self.remove_widget(window_id, event_loop);
                            } else if mx >= mx_x && mx <= mx_x + cw && my >= cy && my <= cy + ch {
                                if widget.config.scale == 1.0 {
                                    widget.config.scale = 2.0;
                                } else {
                                    widget.config.scale = 1.0;
                                }
                                self.app_config.widgets[widget.config_index].scale = widget.config.scale;
                                crate::events::send(crate::events::AppEvent::SaveConfig);
                                widget.reload_image();
                            } else if mx >= mn_x && mx <= mn_x + cw && my >= cy && my <= cy + ch {
                                widget.window.set_visible(false);
                            } else if mx >= layout.border && mx <= layout.window_w - layout.border && my >= layout.title_h && my <= layout.title_h + layout.menu_h {
                                self.active_widget_uid = Some(widget.uid);
                                if let Ok(handle) = widget.window.window_handle() {
                                    if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                                        use muda::ContextMenu;
                                        unsafe {
                                            if mx < layout.border + 32.0 * layout.scale {
                                                self.edit_menu.show_context_menu_for_hwnd(win32.hwnd.get(), None);
                                            } else if mx < layout.border + 72.0 * layout.scale {
                                                self.view_menu.show_context_menu_for_hwnd(win32.hwnd.get(), None);
                                            } else {
                                                self.help_menu.show_context_menu_for_hwnd(win32.hwnd.get(), None);
                                            }
                                        }
                                    }
                                }
                            } else if !widget.config.locked {
                                    if let Some(dir) = get_resize_direction(mx, my, layout.window_w, layout.window_h, layout.border) {
                                        let _ = widget.window.drag_resize_window(dir);
                                    } else {
                                        let _ = widget.window.drag_window();
                                    }
                                }
                            } else if widget.config.layer == Layer::Standard && !widget.config.locked {
                                let _ = widget.window.drag_window();
                            }
                    }
                } else if state == ElementState::Released
                    && button == MouseButton::Left {
                        widget.left_mouse_pressed = false;
                        widget.window.request_redraw();
                    }
            }
            WindowEvent::CursorMoved { position, .. } => {
                widget.cursor_pos = (position.x as f32, position.y as f32);
                if widget.config.shows_chrome() && !widget.config.locked {
                    let size = widget.window.inner_size();
                    let scale_factor = widget.window.scale_factor() as f32;
                    let layout = WidgetLayout::new(
                        &widget.config,
                        size.width as f32,
                        size.height as f32,
                        scale_factor,
                    );
                    if let Some(dir) = get_resize_direction(widget.cursor_pos.0, widget.cursor_pos.1, layout.window_w, layout.window_h, layout.border) {
                        let cursor = match dir {
                            ResizeDirection::East | ResizeDirection::West => CursorIcon::EwResize,
                            ResizeDirection::North | ResizeDirection::South => CursorIcon::NsResize,
                            ResizeDirection::NorthWest | ResizeDirection::SouthEast => CursorIcon::NwseResize,
                            ResizeDirection::NorthEast | ResizeDirection::SouthWest => CursorIcon::NeswResize,
                        };
                        widget.window.set_cursor(cursor);
                    } else {
                        widget.window.set_cursor(CursorIcon::Default);
                    }
                } else {
                    widget.window.set_cursor(CursorIcon::Default);
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                clamp_to_monitor(&widget.window);
                widget.window.request_redraw();
            }
            WindowEvent::Resized(physical_size) => {
                let scale_factor = widget.window.scale_factor();
                let logical_w = physical_size.width as f64 / scale_factor;
                let logical_h = physical_size.height as f64 / scale_factor;
                let s = widget.config.effective_scale() as f64;
                
                let w;
                let mut h;
                let mut chrome_h = 0.0;

                if widget.config.shows_chrome() {
                    let border = crate::render::BORDER_WIDTH as f64;
                    let title = crate::render::TITLE_BAR_HEIGHT as f64;
                    let menu = crate::render::MENU_BAR_HEIGHT as f64;
                    
                    w = logical_w / s - 2.0 * border;
                    chrome_h = title + menu + border;
                    h = logical_h / s - chrome_h;
                } else {
                    w = logical_w / s;
                    h = logical_h / s;
                }
                
                if w > 0.0 && h > 0.0 {
                    if widget.config.fit != crate::config::Fit::Stretch {
                        let intrinsic_size = match &widget.img_state {
                            Some(crate::image_src::ImageState::Static(img)) => Some((img.width() as f64, img.height() as f64)),
                            Some(crate::image_src::ImageState::Animated { frames, .. }) => if !frames.is_empty() { Some((frames[0].width() as f64, frames[0].height() as f64)) } else { None },
                            _ => None,
                        };
                        if let Some((iw, ih)) = intrinsic_size {
                            if iw > 0.0 && ih > 0.0 {
                                let ratio = ih / iw;
                                let expected_h = w * ratio;
                                if (h - expected_h).abs() > 2.0 {
                                    h = expected_h;
                                    let new_logical_h = (h + chrome_h) * s;
                                    let _ = widget.window.request_inner_size(LogicalSize::new(logical_w, new_logical_h));
                                    return;
                                }
                            }
                        }
                    }
                    widget.config.width = w as u32;
                    widget.config.height = h as u32;
                }
                
                self.app_config.widgets[widget.config_index].width = widget.config.width;
                self.app_config.widgets[widget.config_index].height = widget.config.height;
                crate::events::send(crate::events::AppEvent::SaveConfig);
                widget.window.request_redraw();
            }
            WindowEvent::Focused(focused) => {
                widget.is_focused = focused;
                widget.window.request_redraw();
            }
            WindowEvent::Moved(position) => {
                let mut final_x = position.x;
                let mut final_y = position.y;
                
                if let Some(monitor) = widget.window.current_monitor() {
                    let screen_pos = monitor.position();
                    let screen_size = monitor.size();
                    let snap_threshold = 20;
                    
                    if (final_x - screen_pos.x).abs() < snap_threshold {
                        final_x = screen_pos.x;
                    } else {
                        let win_w = widget.window.outer_size().width as i32;
                        if (final_x + win_w - (screen_pos.x + screen_size.width as i32)).abs() < snap_threshold {
                            final_x = screen_pos.x + screen_size.width as i32 - win_w;
                        }
                    }
                    
                    if (final_y - screen_pos.y).abs() < snap_threshold {
                        final_y = screen_pos.y;
                    } else {
                        let win_h = widget.window.outer_size().height as i32;
                        if (final_y + win_h - (screen_pos.y + screen_size.height as i32)).abs() < snap_threshold {
                            final_y = screen_pos.y + screen_size.height as i32 - win_h;
                        }
                    }
                    
                    if final_x != position.x || final_y != position.y {
                        widget.window.set_outer_position(winit::dpi::PhysicalPosition::new(final_x, final_y));
                    }
                }

                widget.config.x = final_x;
                widget.config.y = final_y;
                self.app_config.widgets[widget.config_index].x = final_x;
                self.app_config.widgets[widget.config_index].y = final_y;
                crate::events::send(crate::events::AppEvent::SaveConfig);
            }
            WindowEvent::CursorEntered { .. } => {
                widget.is_hovered = true;
                widget.window.request_redraw();
            }
            WindowEvent::CursorLeft { .. } => {
                widget.is_hovered = false;
                widget.window.request_redraw();
            }
            WindowEvent::DroppedFile(path) => {
                self.app_config.widgets[widget.config_index].image_path =
                    path.to_string_lossy().to_string();
                crate::events::send(crate::events::AppEvent::SaveConfig);
            }
            WindowEvent::RedrawRequested => {
                let size = widget.window.inner_size();
                if size.width == 0 || size.height == 0 {
                    return;
                }

                widget
                    .surface
                    .resize(
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    )
                    .unwrap();

                let mut buffer = widget.surface.buffer_mut().unwrap();
                let mut pixmap = tiny_skia::Pixmap::new(size.width, size.height).unwrap();

                let scale_factor = widget.window.scale_factor() as f32;
                let layout = WidgetLayout::new(
                    &widget.config,
                    size.width as f32,
                    size.height as f32,
                    scale_factor,
                );

                // menu_open and menu_pos are removed from RenderState draw since we use native menu
                widget.render_state.draw(
                    &mut pixmap,
                    &widget.config,
                    &widget.img_state,
                    widget.cursor_pos.0,
                    widget.cursor_pos.1,
                    widget.left_mouse_pressed,
                    widget.is_focused,
                    widget.is_hovered,
                    &layout,
                );

                for (i, pixel) in pixmap.pixels().iter().enumerate() {
                    let r = pixel.red() as u32;
                    let g = pixel.green() as u32;
                    let b = pixel.blue() as u32;
                    let a = pixel.alpha() as u32;
                    buffer[i] = b | (g << 8) | (r << 16) | (a << 24);
                }
                buffer.present().unwrap();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let mut min_wait: Option<std::time::Instant> = None;
        let now = std::time::Instant::now();
        
        for widget in self.widgets.values_mut() {
            if widget.config.interval > 0 {
                let interval_dur = std::time::Duration::from_secs(widget.config.interval as u64);
                let next_time = widget.last_slide_time + interval_dur;
                
                if now >= next_time {
                    widget.last_slide_time = now;
                    if widget.config.random {
                        widget.source.random();
                    } else {
                        widget.source.next();
                    }
                    widget.begin_load();
                    
                    let next_next = now + interval_dur;
                    min_wait = Some(min_wait.map_or(next_next, |m| m.min(next_next)));
                } else {
                    min_wait = Some(min_wait.map_or(next_time, |m| m.min(next_time)));
                }
            }

            if let Some(crate::image_src::ImageState::Animated { frames, delays, current_frame, last_frame_time }) = &mut widget.img_state {
                let delay = std::time::Duration::from_millis(delays[*current_frame] as u64);
                let next_frame_time = *last_frame_time + delay;
                
                if now >= next_frame_time {
                    *current_frame = (*current_frame + 1) % frames.len();
                    *last_frame_time = now;
                    widget.window.request_redraw();
                    
                    let new_delay = std::time::Duration::from_millis(delays[*current_frame] as u64);
                    let next_next = now + new_delay;
                    min_wait = Some(min_wait.map_or(next_next, |m| m.min(next_next)));
                } else {
                    min_wait = Some(min_wait.map_or(next_frame_time, |m| m.min(next_frame_time)));
                }
            }
        }
        
        if let Some(wait) = min_wait {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(wait));
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        }
    }
}

fn get_resize_direction(x: f32, y: f32, w: f32, h: f32, border: f32) -> Option<ResizeDirection> {
    let left = x < border;
    let right = x > w - border;
    let top = y < border;
    let bottom = y > h - border;

    if left && top {
        Some(ResizeDirection::NorthWest)
    } else if right && top {
        Some(ResizeDirection::NorthEast)
    } else if left && bottom {
        Some(ResizeDirection::SouthWest)
    } else if right && bottom {
        Some(ResizeDirection::SouthEast)
    } else if left {
        Some(ResizeDirection::West)
    } else if right {
        Some(ResizeDirection::East)
    } else if top {
        Some(ResizeDirection::North)
    } else if bottom {
        Some(ResizeDirection::South)
    } else {
        None
    }
}
