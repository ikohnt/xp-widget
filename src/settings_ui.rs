use native_windows_gui as nwg;
use native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct SettingsApp {
    #[nwg_control(size: (350, 350), position: (300, 300), title: "XP Widget Settings", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [SettingsApp::exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, margin: [10, 10, 10, 10], spacing: 5)]
    layout: nwg::GridLayout,

    #[nwg_control(text: "Theme:")]
    #[nwg_layout_item(layout: layout, col: 0, row: 0)]
    theme_label: nwg::Label,

    #[nwg_control(collection: vec!["Classic".to_string(), "Win98".to_string(), "Aero Glass".to_string(), "MacOS Aqua".to_string()])]
    #[nwg_layout_item(layout: layout, col: 1, row: 0)]
    theme_combo: nwg::ComboBox<String>,

    #[nwg_control(text: "Opacity (0-1):")]
    #[nwg_layout_item(layout: layout, col: 0, row: 1)]
    opacity_label: nwg::Label,

    #[nwg_control(text: "1.0")]
    #[nwg_layout_item(layout: layout, col: 1, row: 1)]
    opacity_input: nwg::TextInput,

    #[nwg_control(text: "Scale (x):")]
    #[nwg_layout_item(layout: layout, col: 0, row: 2)]
    scale_label: nwg::Label,

    #[nwg_control(text: "1.0")]
    #[nwg_layout_item(layout: layout, col: 1, row: 2)]
    scale_input: nwg::TextInput,

    #[nwg_control(text: "Width (px):")]
    #[nwg_layout_item(layout: layout, col: 0, row: 3)]
    width_label: nwg::Label,

    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: layout, col: 1, row: 3)]
    width_input: nwg::TextInput,

    #[nwg_control(text: "Height (px):")]
    #[nwg_layout_item(layout: layout, col: 0, row: 4)]
    height_label: nwg::Label,

    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: layout, col: 1, row: 4)]
    height_input: nwg::TextInput,

    #[nwg_control(text: "Slideshow Interval (s):")]
    #[nwg_layout_item(layout: layout, col: 0, row: 5)]
    interval_label: nwg::Label,

    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: layout, col: 1, row: 5)]
    interval_input: nwg::TextInput,

    #[nwg_control(text: "Ghost Mode (Click-through)")]
    #[nwg_layout_item(layout: layout, col: 0, row: 6)]
    ghost_mode_check: nwg::CheckBox,
    
    #[nwg_control(text: "Rounded Corners")]
    #[nwg_layout_item(layout: layout, col: 1, row: 6)]
    rounded_corners_check: nwg::CheckBox,

    #[nwg_control(text: "Save & Apply")]
    #[nwg_layout_item(layout: layout, col: 1, row: 7)]
    #[nwg_events( OnButtonClick: [SettingsApp::save] )]
    save_button: nwg::Button,
}

impl SettingsApp {
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn save(&self) {
        let mut config = crate::config::load_or_create();
            if let Some(theme_display) = self.theme_combo.selection_string() {
                let theme_id = match theme_display.as_str() {
                    "Win98" => "98",
                    "Aero Glass" => "aero",
                    "MacOS Aqua" => "aqua",
                    _ => "classic", // Default for "Classic" or unknown
                };
                for w in config.config.widgets.iter_mut() {
                    w.theme = theme_id.to_string();
                }
            }
            if let Ok(op) = self.opacity_input.text().parse::<f32>() {
                for w in config.config.widgets.iter_mut() {
                    w.opacity = op;
                }
            }
            if let Ok(sc) = self.scale_input.text().parse::<f32>() {
                for w in config.config.widgets.iter_mut() {
                    w.scale = sc;
                }
            }
            if let Ok(w_val) = self.width_input.text().parse::<u32>() {
                for w in config.config.widgets.iter_mut() {
                    w.width = w_val;
                }
            }
            if let Ok(h_val) = self.height_input.text().parse::<u32>() {
                for w in config.config.widgets.iter_mut() {
                    w.height = h_val;
                }
            }
            if let Ok(i_val) = self.interval_input.text().parse::<u32>() {
                for w in config.config.widgets.iter_mut() {
                    w.interval = i_val;
                }
            }
            
            let is_ghost = self.ghost_mode_check.check_state() == nwg::CheckBoxState::Checked;
            let is_rounded = self.rounded_corners_check.check_state() == nwg::CheckBoxState::Checked;
            for w in config.config.widgets.iter_mut() {
                w.ghost_mode = is_ghost;
                w.rounded_corners = is_rounded;
            }
            
            let text = toml::to_string(&config.config).unwrap_or_default();
            if let Some(cfg_path) = crate::config::AppConfig::path() {
                let _ = std::fs::write(cfg_path, text);
            }
        self.exit();
    }
}

pub fn open_settings() {
    std::thread::spawn(|| {
        nwg::init().expect("Failed to init Native Windows GUI");
        let app = SettingsApp::build_ui(Default::default()).expect("Failed to build UI");
        
        // Pre-fill current config
        let config = crate::config::load_or_create();
        if let Some(w) = config.config.widgets.first() {
            app.opacity_input.set_text(&w.opacity.to_string());
            app.scale_input.set_text(&w.scale.to_string());
            app.width_input.set_text(&w.width.to_string());
            app.height_input.set_text(&w.height.to_string());
            app.interval_input.set_text(&w.interval.to_string());
            
            // Set combo box selection if matches
            let combo_str = match w.theme.as_str() {
                "98" => "Win98",
                "aero" => "Aero Glass",
                "aqua" => "MacOS Aqua",
                _ => "Classic",
            };
            if let Some(idx) = app.theme_combo.collection().iter().position(|t| t == combo_str) {
                app.theme_combo.set_selection(Some(idx));
            }
            
            if w.ghost_mode {
                app.ghost_mode_check.set_check_state(nwg::CheckBoxState::Checked);
            } else {
                app.ghost_mode_check.set_check_state(nwg::CheckBoxState::Unchecked);
            }
            if w.rounded_corners {
                app.rounded_corners_check.set_check_state(nwg::CheckBoxState::Checked);
            } else {
                app.rounded_corners_check.set_check_state(nwg::CheckBoxState::Unchecked);
            }
        }
        
        nwg::dispatch_thread_events();
    });
}
