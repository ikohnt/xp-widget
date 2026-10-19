use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::render::{BORDER_WIDTH, TITLE_BAR_HEIGHT};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Layer {
    #[default]
    Standard,
    Desktop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum Fit {
    #[default]
    Fit,
    Stretch,
    Tile,
    Contain,
    Cover,
    Fill,
    None,
    #[serde(rename = "scale-down")]
    ScaleDown,
}

/// Settings for a single widget window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WidgetConfig {
    pub image_path: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub background_color: String,
    pub layer: Layer,
    pub chrome: bool,
    pub window_title: String,
    pub fit: Fit,
    pub brightness: f32,
    pub opacity: f32,
    pub rotation: i32,
    pub frame: bool,
    pub locked: bool,
    pub interval: u32,
    pub random: bool,
    pub theme: String,
    pub alt_text: String,
    pub ghost_mode: bool,
    pub rounded_corners: bool,
}

impl Default for WidgetConfig {
    fn default() -> Self {
        Self {
            image_path: String::new(),
            x: 100,
            y: 100,
            width: 400,
            height: 300,
            scale: 1.0,
            background_color: "#000000".to_string(),
            layer: Layer::Standard,
            chrome: true,
            window_title: "XP Widget".to_string(),
            fit: Fit::Fit,
            brightness: 1.0,
            opacity: 1.0,
            rotation: 0,
            frame: false,
            locked: false,
            interval: 0,
            random: false,
            theme: "95".to_string(),
            alt_text: String::new(),
            ghost_mode: false,
            rounded_corners: true,
        }
    }
}

impl WidgetConfig {
    /// `scale = 0.0` means "auto", which currently behaves like 1.0.
    pub fn effective_scale(&self) -> f32 {
        if self.scale <= 0.0 {
            1.0
        } else {
            self.scale
        }
    }

    /// The XP title bar/border is only drawn on the standard (floating) layer.
    pub fn shows_chrome(&self) -> bool {
        self.chrome && self.layer == Layer::Standard
    }

    /// Logical window size. `width`/`height` describe the image area, so the
    /// XP chrome is added around it (as documented in the README).
    pub fn window_logical_size(&self) -> (f64, f64) {
        let s = self.effective_scale() as f64;
        let (w, h) = (self.width as f64, self.height as f64);
        if self.shows_chrome() {
            let border = BORDER_WIDTH as f64;
            let title = TITLE_BAR_HEIGHT as f64;
            let menu = crate::render::MENU_BAR_HEIGHT as f64;
            ((w + 2.0 * border) * s, (h + title + menu + border) * s)
        } else {
            (w * s, h * s)
        }
    }

    pub fn validate_and_fallback(&mut self) {
        if self.scale < 0.0 {
            self.scale = 0.0;
        } else if self.scale > 0.0 && self.scale < 0.5 {
            crate::log::error("scale < 0.5, falling back to 0.5");
            self.scale = 0.5;
        } else if self.scale > 4.0 {
            crate::log::error("scale > 4.0, falling back to 4.0");
            self.scale = 4.0;
        }

        self.opacity = self.opacity.clamp(0.0, 1.0);
        self.brightness = self.brightness.clamp(0.0, 4.0);
        self.width = self.width.clamp(50, 8000);
        self.height = self.height.clamp(50, 8000);

        if self.rotation != 0 && self.rotation != 90 && self.rotation != 180 && self.rotation != 270
        {
            crate::log::error("Invalid rotation, falling back to 0");
            self.rotation = 0;
        }
    }
}

/// The whole config file: global options plus one entry per widget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub verbose: bool,
    pub widgets: Vec<WidgetConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            widgets: vec![WidgetConfig::default()],
        }
    }
}

const FILE_HEADER: &str = r##"# xp-widget configuration
#
# Tip: you rarely need to edit this by hand. Right-click the tray icon (or a
# widget) and choose "Settings...". This file is watched, so saving it here
# applies your changes immediately.
#
# Each [[widgets]] block is one widget window:
#   image_path        image file (png/jpeg/gif) or a folder of images (use the
#                     arrow keys on the widget to browse). Prefer single quotes
#                     for Windows paths: 'C:\Users\me\Pictures\cat.png'
#   x, y              top-left position (logical pixels)
#   width, height     size of the image area (logical pixels, excluding the XP frame)
#   scale             size multiplier, 0.5 - 4.0
#   background_color  hex color behind letterboxing / faded pixels, e.g. "#000000"
#   layer             "standard" (floating window) or "desktop" (behind icons, click-through)
#   chrome            true = Windows XP title bar and border, false = bare image
#   window_title      text shown in the title bar
#   fit               "fit", "stretch" or "tile"
#   brightness        1.0 = unchanged
#   opacity           0.0 - 1.0 (fades the image into background_color)
#   rotation          0, 90, 180 or 270
#
# verbose = true writes extra details to widget.log in this folder.
"##;

impl AppConfig {
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("xp-widget");
            p.push("config.toml");
            p
        })
    }

    /// Parses either the current multi-widget format or the legacy (v0.1)
    /// single-widget format, which is migrated transparently.
    pub fn parse(text: &str) -> Result<Self, String> {
        let table: toml::Table = text.parse().map_err(|e| describe_error(&e, text))?;

        let mut cfg = if table.contains_key("widgets") {
            toml::from_str::<AppConfig>(text).map_err(|e| describe_error(&e, text))?
        } else {
            // Legacy file: the widget fields live at the top level. An empty
            // (or fully commented-out) file also lands here and yields one
            // default widget.
            let widget: WidgetConfig =
                toml::from_str(text).map_err(|e| describe_error(&e, text))?;
            let verbose = table
                .get("verbose")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            AppConfig {
                verbose,
                widgets: vec![widget],
            }
        };

        for w in &mut cfg.widgets {
            w.validate_and_fallback();
        }
        Ok(cfg)
    }

    pub fn to_file_string(&self) -> String {
        let body = toml::to_string_pretty(self).unwrap_or_default();
        format!("{FILE_HEADER}\n{body}")
    }
}

/// Short, human readable parse error ("line 3: invalid string").
fn describe_error(e: &toml::de::Error, text: &str) -> String {
    let msg = e.message().trim().to_string();
    match e.span() {
        Some(span) => {
            let line = text[..span.start.min(text.len())].matches('\n').count() + 1;
            format!("line {line}: {msg}")
        }
        None => msg,
    }
}

#[allow(dead_code)]
pub struct Loaded {
    pub config: AppConfig,
    /// Raw file contents, used to ignore change notifications for our own saves.
    pub text: String,
    pub first_run: bool,
    pub error: Option<String>,
}

/// Loads config.toml, creating a documented default file on first run.
pub fn load_or_create() -> Loaded {
    let Some(path) = AppConfig::path() else {
        return Loaded {
            config: AppConfig::default(),
            text: String::new(),
            first_run: false,
            error: None,
        };
    };

    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let config = AppConfig::default();
        let text = config.to_file_string();
        if let Err(e) = std::fs::write(&path, &text) {
            crate::log::error(&format!("Could not create {}: {e}", path.display()));
        }
        return Loaded {
            config,
            text,
            first_run: true,
            error: None,
        };
    }

    let text = std::fs::read_to_string(&path).unwrap_or_default();
    match AppConfig::parse(&text) {
        Ok(config) => Loaded {
            config,
            text,
            first_run: false,
            error: None,
        },
        Err(e) => {
            crate::log::error(&format!("Failed to parse config.toml: {e}"));
            Loaded {
                config: AppConfig::default(),
                text,
                first_run: false,
                error: Some(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_resolution() {
        let mut cfg = WidgetConfig::default();

        cfg.scale = -1.0;
        cfg.validate_and_fallback();
        assert_eq!(cfg.scale, 0.0);
        assert_eq!(cfg.effective_scale(), 1.0);

        cfg.scale = 0.1;
        cfg.validate_and_fallback();
        assert_eq!(cfg.scale, 0.5);

        cfg.scale = 5.0;
        cfg.validate_and_fallback();
        assert_eq!(cfg.scale, 4.0);

        cfg.scale = 2.0;
        cfg.validate_and_fallback();
        assert_eq!(cfg.scale, 2.0);
    }

    #[test]
    fn test_fallback_invalid_values() {
        let mut cfg = WidgetConfig::default();
        cfg.opacity = 2.0;
        cfg.rotation = 45;

        cfg.validate_and_fallback();

        assert_eq!(cfg.layer, Layer::Standard);
        assert_eq!(cfg.fit, Fit::Fit);
        assert_eq!(cfg.opacity, 1.0);
        assert_eq!(cfg.rotation, 0);
    }

    #[test]
    fn test_legacy_single_widget_migrates() {
        let legacy = "image_path = 'C:\\pics\\cat.png'\nx = 10\ny = 20\nverbose = true\n";
        let cfg = AppConfig::parse(legacy).unwrap();
        assert!(cfg.verbose);
        assert_eq!(cfg.widgets.len(), 1);
        assert_eq!(cfg.widgets[0].image_path, "C:\\pics\\cat.png");
        assert_eq!((cfg.widgets[0].x, cfg.widgets[0].y), (10, 20));
    }

    #[test]
    fn test_commented_out_file_gives_default_widget() {
        let cfg = AppConfig::parse("# x = 100\n# y = 100\n").unwrap();
        assert_eq!(cfg.widgets, vec![WidgetConfig::default()]);
    }

    #[test]
    fn test_multi_widget_round_trip() {
        let mut cfg = AppConfig::default();
        cfg.widgets[0].image_path = "C:\\Users\\me\\a.png".into();
        cfg.widgets.push(WidgetConfig {
            x: 500,
            layer: Layer::Desktop,
            ..Default::default()
        });
        let text = cfg.to_file_string();
        assert_eq!(AppConfig::parse(&text).unwrap(), cfg);
    }

    #[test]
    fn test_parse_error_mentions_line() {
        let err = AppConfig::parse("x = 1\nimage_path = \"\"C:\\bad\"\"\n").unwrap_err();
        assert!(err.starts_with("line 2"), "{err}");
    }

    #[test]
    fn test_window_size_includes_chrome() {
        let cfg = WidgetConfig::default();
        let (w, h) = cfg.window_logical_size();
        assert_eq!(w, 400.0 + 2.0 * BORDER_WIDTH as f64);
        assert_eq!(h, 300.0 + TITLE_BAR_HEIGHT as f64 + crate::render::MENU_BAR_HEIGHT as f64 + BORDER_WIDTH as f64);
        let bare = WidgetConfig {
            chrome: false,
            ..Default::default()
        };
        assert_eq!(bare.window_logical_size(), (400.0, 300.0));
    }
}
