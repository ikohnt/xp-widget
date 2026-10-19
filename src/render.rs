#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::too_many_arguments)]

use crate::config::{Fit, WidgetConfig};
use crate::image_src::ImageState;
use tiny_skia::*;

const FONT_DATA: &[u8] = include_bytes!("font.ttf");
pub const TITLE_BAR_HEIGHT: f32 = 24.0;
pub const MENU_BAR_HEIGHT: f32 = 20.0;
pub const BORDER_WIDTH: f32 = 4.0;
pub const MENU_WIDTH: f32 = 120.0;
pub const MENU_ITEM_HEIGHT: f32 = 24.0;

#[derive(Debug, Clone, Copy)]
pub struct WidgetLayout {
    pub scale: f32,
    pub border: f32,
    pub title_h: f32,
    pub menu_h: f32,
    pub pad: f32,
    pub window_w: f32,
    pub window_h: f32,
    pub is_aqua: bool,
}

impl WidgetLayout {
    pub fn new(config: &WidgetConfig, window_w: f32, window_h: f32, scale_factor: f32) -> Self {
        let scale = config.effective_scale() * scale_factor;
        let shows_chrome = config.shows_chrome();

        let border = if shows_chrome {
            BORDER_WIDTH * scale
        } else {
            0.0
        };
        let title_h = if shows_chrome {
            TITLE_BAR_HEIGHT * scale
        } else {
            0.0
        };
        let menu_h = if shows_chrome {
            MENU_BAR_HEIGHT * scale
        } else {
            0.0
        };
        let pad = if shows_chrome { 2.0 * scale } else { 0.0 };

        Self {
            scale,
            border,
            title_h,
            menu_h,
            pad,
            window_w,
            window_h,
            is_aqua: config.theme == "aqua",
        }
    }

    pub fn close_button_rect(&self) -> (f32, f32, f32, f32) {
        let btn_w = 16.0 * self.scale;
        let btn_pad = 2.0 * self.scale;
        let btn_y = self.border + btn_pad;
        let close_x = if self.is_aqua {
            self.border + btn_pad
        } else {
            self.window_w - self.border - btn_pad - btn_w
        };
        (close_x, btn_y, btn_w, btn_w)
    }

    pub fn content_rect(&self) -> Rect {
        let content_y = self.title_h + self.menu_h;
        snap_xywh(
            self.border + self.pad,
            content_y + self.pad,
            self.window_w - 2.0 * self.border - 2.0 * self.pad,
            self.window_h - content_y - self.border - 2.0 * self.pad,
        )
        .unwrap_or(snap_xywh(0.0, 0.0, self.window_w, self.window_h).unwrap())
    }
}

pub fn snap_xywh(x: f32, y: f32, w: f32, h: f32) -> Option<Rect> {
    Rect::from_xywh(x.round(), y.round(), w.round(), h.round())
}

pub struct RenderState {
    font: fontdue::Font,
}

impl Default for RenderState {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderState {
    pub fn new() -> Self {
        let font = fontdue::Font::from_bytes(FONT_DATA, fontdue::FontSettings::default()).unwrap();
        Self { font }
    }

    pub fn draw(
        &self,
        pixmap: &mut Pixmap,
        config: &WidgetConfig,
        img_state: &Option<ImageState>,
        mx: f32,
        my: f32,
        pressed: bool,
        is_focused: bool,
        is_hovered: bool,
        layout: &WidgetLayout,
    ) {
        pixmap.fill(Color::TRANSPARENT);

        let scale = layout.scale;

        let content_rect = if config.shows_chrome() {
            self.draw_chrome(pixmap, layout, config, mx, my, pressed, is_focused);
            layout.content_rect()
        } else {
            snap_xywh(0.0, 0.0, layout.window_w, layout.window_h).unwrap()
        };

        let mut actual_content = content_rect;

        if config.frame {
            // Draw a subtle shadow and a polaroid white frame
            let mut shadow = Paint::default();
            shadow.set_color_rgba8(0, 0, 0, 60);
            if let Some(shadow_rect) = snap_xywh(
                content_rect.x() + 2.0 * scale,
                content_rect.y() + 4.0 * scale,
                content_rect.width(),
                content_rect.height(),
            ) {
                pixmap.fill_rect(shadow_rect, &shadow, Transform::identity(), None);
            }

            let mut frame_paint = Paint::default();
            frame_paint.set_color_rgba8(250, 250, 250, 255);
            pixmap.fill_rect(content_rect, &frame_paint, Transform::identity(), None);

            let border = 8.0 * scale;
            let bottom_border = 32.0 * scale;

            actual_content = snap_xywh(
                content_rect.x() + border,
                content_rect.y() + border,
                content_rect.width() - 2.0 * border,
                content_rect.height() - border - bottom_border,
            )
            .unwrap_or(content_rect);
        } else if let Some(bg_color) = parse_hex_color(&config.background_color) {
            let mut bg_paint = Paint::default();
            bg_paint.set_color(bg_color);
            pixmap.fill_rect(content_rect, &bg_paint, Transform::identity(), None);
        }

        if let Some(state) = img_state {
            match state {
                ImageState::Static(img) => {
                    self.draw_image(pixmap, img, actual_content, &config.fit, config.rounded_corners);
                }
                ImageState::Animated { frames, current_frame, .. } => {
                    if *current_frame < frames.len() {
                        self.draw_image(pixmap, &frames[*current_frame], actual_content, &config.fit, config.rounded_corners);
                    }
                }
                ImageState::Error(msg) => {
                    self.draw_placeholder(pixmap, actual_content, msg, scale);
                }
                ImageState::Loading => {
                    self.draw_placeholder(pixmap, actual_content, "Loading...", scale);
                }
            }
        } else {
            self.draw_placeholder(pixmap, actual_content, "No image", scale);
        }
        if is_hovered && !config.shows_chrome() {
            let mut overlay = Paint::default();
            overlay.set_color_rgba8(0, 0, 0, 120);
            if let Some(top_rect) = snap_xywh(content_rect.x(), content_rect.y(), content_rect.width(), 20.0 * scale) {
                pixmap.fill_rect(top_rect, &overlay, Transform::identity(), None);
                self.draw_text(
                    pixmap,
                    "Hold Left-Click to drag. Right-Click for menu.",
                    content_rect.x() + 4.0 * scale,
                    content_rect.y() + 4.0 * scale,
                    11.0 * scale,
                    Color::from_rgba8(255, 255, 255, 255),
                    false,
                );
            }
        }
    }

    fn draw_chrome(
        &self,
        pixmap: &mut Pixmap,
        layout: &WidgetLayout,
        config: &WidgetConfig,
        mx: f32,
        my: f32,
        pressed: bool,
        is_focused: bool,
    ) {
        let title_h = layout.title_h;
        let menu_h = layout.menu_h;
        let border = layout.border;
        let w = layout.window_w;
        let h = layout.window_h;
        let scale = layout.scale;

        // Base light gray fill for the entire window
        let mut bg_paint = Paint::default();
        if config.theme == "aero" {
            bg_paint.set_color_rgba8(220, 230, 255, 100); // Translucent light blue
        } else {
            bg_paint.set_color_rgba8(192, 192, 192, 255); // Classic light gray
        }
        pixmap.fill_rect(
            snap_xywh(0.0, 0.0, w, h).unwrap(),
            &bg_paint,
            Transform::identity(),
            None,
        );

        // 3D Outer Border (Raised)
        // White top/left
        let mut white = Paint::default();
        white.set_color_rgba8(255, 255, 255, 255);
        pixmap.fill_rect(
            snap_xywh(0.0, 0.0, w, 1.0 * scale).unwrap(),
            &white,
            Transform::identity(),
            None,
        );
        pixmap.fill_rect(
            snap_xywh(0.0, 0.0, 1.0 * scale, h).unwrap(),
            &white,
            Transform::identity(),
            None,
        );

        // Black bottom/right
        let mut black = Paint::default();
        black.set_color_rgba8(0, 0, 0, 255);
        pixmap.fill_rect(
            snap_xywh(0.0, h - 1.0 * scale, w, 1.0 * scale).unwrap(),
            &black,
            Transform::identity(),
            None,
        );
        pixmap.fill_rect(
            snap_xywh(w - 1.0 * scale, 0.0, 1.0 * scale, h).unwrap(),
            &black,
            Transform::identity(),
            None,
        );

        // Dark gray bottom/right (inner shadow)
        let mut dark_gray = Paint::default();
        dark_gray.set_color_rgba8(128, 128, 128, 255);
        pixmap.fill_rect(
            snap_xywh(1.0 * scale, h - 2.0 * scale, w - 2.0 * scale, 1.0 * scale).unwrap(),
            &dark_gray,
            Transform::identity(),
            None,
        );
        pixmap.fill_rect(
            snap_xywh(w - 2.0 * scale, 1.0 * scale, 1.0 * scale, h - 2.0 * scale).unwrap(),
            &dark_gray,
            Transform::identity(),
            None,
        );

        // Title bar (Navy Blue if focused, Dark Gray if unfocused)
        let title_rect = snap_xywh(border, border, w - 2.0 * border, title_h - border).unwrap();

        let mut title_paint = Paint::default();
        if config.theme == "98" {
            let start = Point::from_xy(title_rect.x(), title_rect.y());
            let end = Point::from_xy(title_rect.x() + title_rect.width(), title_rect.y());
            let stops = if is_focused {
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(0, 0, 170, 255)), // Navy
                    GradientStop::new(1.0, Color::from_rgba8(16, 132, 208, 255)), // Light Blue
                ]
            } else {
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(128, 128, 128, 255)), // Dark Gray
                    GradientStop::new(1.0, Color::from_rgba8(192, 192, 192, 255)), // Light Gray
                ]
            };
            if let Some(shader) = LinearGradient::new(start, end, stops, SpreadMode::Pad, Transform::identity()) {
                title_paint.shader = shader;
            } else {
                title_paint.set_color_rgba8(0, 0, 170, 255);
            }
        } else if config.theme == "aero" {
            let start = Point::from_xy(title_rect.x(), title_rect.y());
            let end = Point::from_xy(title_rect.x(), title_rect.y() + title_rect.height());
            let stops = if is_focused {
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(255, 255, 255, 150)),
                    GradientStop::new(0.5, Color::from_rgba8(200, 220, 255, 80)),
                    GradientStop::new(1.0, Color::from_rgba8(150, 180, 220, 160)),
                ]
            } else {
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(255, 255, 255, 80)),
                    GradientStop::new(0.5, Color::from_rgba8(220, 220, 220, 50)),
                    GradientStop::new(1.0, Color::from_rgba8(180, 180, 180, 100)),
                ]
            };
            if let Some(shader) = LinearGradient::new(start, end, stops, SpreadMode::Pad, Transform::identity()) {
                title_paint.shader = shader;
            } else {
                title_paint.set_color_rgba8(200, 220, 255, 100);
            }
        } else if config.theme == "aqua" {
            let start = Point::from_xy(title_rect.x(), title_rect.y());
            let end = Point::from_xy(title_rect.x(), title_rect.y() + title_rect.height());
            let stops = if is_focused {
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(245, 245, 245, 255)),
                    GradientStop::new(1.0, Color::from_rgba8(200, 200, 200, 255)),
                ]
            } else {
                vec![
                    GradientStop::new(0.0, Color::from_rgba8(235, 235, 235, 255)),
                    GradientStop::new(1.0, Color::from_rgba8(210, 210, 210, 255)),
                ]
            };
            if let Some(shader) = LinearGradient::new(start, end, stops, SpreadMode::Pad, Transform::identity()) {
                title_paint.shader = shader;
            } else {
                title_paint.set_color_rgba8(220, 220, 220, 255);
            }
        } else {
            if is_focused {
                title_paint.set_color_rgba8(0, 0, 170, 255); // Classic Navy
            } else {
                title_paint.set_color_rgba8(128, 128, 128, 255); // Inactive Gray
            }
        }
        
        pixmap.fill_rect(title_rect, &title_paint, Transform::identity(), None);

        let display_title = if config.locked {
            format!("[Locked] {}", config.window_title)
        } else {
            config.window_title.clone()
        };

        // Draw title text (centered for aqua, left for classic/aero)
        let display_title_color = if config.theme == "aqua" {
            if is_focused { Color::from_rgba8(50, 50, 50, 255) } else { Color::from_rgba8(120, 120, 120, 255) }
        } else {
            if is_focused { Color::WHITE } else { Color::from_rgba8(212, 208, 200, 255) }
        };

        let title_x = if config.theme == "aqua" {
            // Roughly center the title
            let text_w = display_title.len() as f32 * 6.0 * scale; // Approximation
            (w - text_w) / 2.0
        } else {
            border + 4.0 * scale
        };

        self.draw_text(
            pixmap,
            &display_title,
            title_x,
            border + (title_h - border - 12.0 * scale) / 2.0,
            12.0 * scale,
            display_title_color,
            true,
        );

        // Menu Bar background and text
        let menu_y = title_h;
        if config.theme == "aqua" {
            // Aqua has a single continuous metal look, no distinct menu bar color needed
        } else {
            self.draw_text(
                pixmap,
                "Edit  View  Help",
                border + 4.0 * scale,
                menu_y + (menu_h - 12.0 * scale) / 2.0,
                12.0 * scale,
                Color::BLACK,
                false,
            );
        }

        // 3D Inner Border (Sunken) around content area
        let content_y = title_h + menu_h;
        let content_h = h - content_y - border;
        let content_w = w - 2.0 * border;

        // Top/Left dark gray (or translucent if aero)
        if config.theme == "aero" {
            dark_gray.set_color_rgba8(0, 0, 0, 80);
        }
        pixmap.fill_rect(
            snap_xywh(border, content_y, content_w, 1.0 * scale).unwrap(),
            &dark_gray,
            Transform::identity(),
            None,
        );
        pixmap.fill_rect(
            snap_xywh(border, content_y, 1.0 * scale, content_h).unwrap(),
            &dark_gray,
            Transform::identity(),
            None,
        );
        // Top/Left black (inner)
        pixmap.fill_rect(
            snap_xywh(
                border + 1.0 * scale,
                content_y + 1.0 * scale,
                content_w - 2.0 * scale,
                1.0 * scale,
            )
            .unwrap(),
            &black,
            Transform::identity(),
            None,
        );
        pixmap.fill_rect(
            snap_xywh(
                border + 1.0 * scale,
                content_y + 1.0 * scale,
                1.0 * scale,
                content_h - 2.0 * scale,
            )
            .unwrap(),
            &black,
            Transform::identity(),
            None,
        );

        // Bottom/Right white
        pixmap.fill_rect(
            snap_xywh(
                border,
                content_y + content_h - 1.0 * scale,
                content_w,
                1.0 * scale,
            )
            .unwrap(),
            &white,
            Transform::identity(),
            None,
        );
        pixmap.fill_rect(
            snap_xywh(
                border + content_w - 1.0 * scale,
                content_y,
                1.0 * scale,
                content_h,
            )
            .unwrap(),
            &white,
            Transform::identity(),
            None,
        );

        // Classic Buttons (Close, Max, Min)
        let (close_x, btn_y, btn_w, _) = layout.close_button_rect();
        
        let (max_x, min_x) = if layout.is_aqua {
            let space = 4.0 * scale;
            (close_x + 2.0 * (btn_w + space), close_x + btn_w + space)
        } else {
            (close_x - btn_w - 2.0 * scale, close_x - 2.0 * btn_w - 4.0 * scale)
        };

        let close_hover = mx >= close_x && mx <= close_x + btn_w && my >= btn_y && my <= btn_y + btn_w;
        let max_hover = mx >= max_x && mx <= max_x + btn_w && my >= btn_y && my <= btn_y + btn_w;
        let min_hover = mx >= min_x && mx <= min_x + btn_w && my >= btn_y && my <= btn_y + btn_w;

        let close_pressed = close_hover && pressed;
        let max_pressed = max_hover && pressed;
        let min_pressed = min_hover && pressed;

        if layout.is_aqua {
            let mut draw_aqua_btn = |x: f32, color: (u8, u8, u8), symbol: &str| {
                let mut paint = Paint::default();
                paint.set_color_rgba8(color.0, color.1, color.2, 255);
                paint.anti_alias = true;
                let cx = x + btn_w / 2.0;
                let cy = btn_y + btn_w / 2.0;
                let r = btn_w / 2.0 - 2.0 * scale;
                let mut pb = PathBuilder::new();
                pb.push_circle(cx, cy, r);
                if let Some(path) = pb.finish() {
                    pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
                }
                
                // Draw inner symbol if hovered
                if (x == close_x && close_hover) || (x == min_x && min_hover) || (x == max_x && max_hover) {
                    self.draw_text(
                        pixmap,
                        symbol,
                        x + 4.0 * scale,
                        btn_y + 1.0 * scale,
                        12.0 * scale,
                        Color::from_rgba8(50, 50, 50, 200),
                        true,
                    );
                }
            };
            draw_aqua_btn(close_x, (255, 95, 86), "x"); // Red
            draw_aqua_btn(min_x, (255, 189, 46), "-");  // Yellow
            draw_aqua_btn(max_x, (39, 201, 63), "+");   // Green
        } else {
            // Close button
            self.draw_classic_button(pixmap, close_x, btn_y, btn_w, btn_w, scale, close_pressed);
            let c_off = if close_pressed { 1.0 * scale } else { 0.0 };
            self.draw_text(
                pixmap,
                "X",
                close_x + 4.0 * scale + c_off,
                btn_y + 2.0 * scale + c_off,
                12.0 * scale,
                Color::BLACK,
                true,
            );

            // Maximize button
            self.draw_classic_button(pixmap, max_x, btn_y, btn_w, btn_w, scale, max_pressed);
            let mx_off = if max_pressed { 1.0 * scale } else { 0.0 };
            self.draw_text(
                pixmap,
                "□",
                max_x + 3.0 * scale + mx_off,
                btn_y + 1.0 * scale + mx_off,
                14.0 * scale,
                Color::BLACK,
                true,
            );

            // Minimize button
            self.draw_classic_button(pixmap, min_x, btn_y, btn_w, btn_w, scale, min_pressed);
            let mn_off = if min_pressed { 1.0 * scale } else { 0.0 };
            self.draw_text(
                pixmap,
                "_",
                min_x + 3.0 * scale + mn_off,
                btn_y - 4.0 * scale + mn_off,
                14.0 * scale,
                Color::BLACK,
                true,
            );
        }
    }

    fn draw_classic_button(&self, pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, scale: f32, pressed: bool) {
        let mut bg = Paint::default();
        bg.set_color_rgba8(192, 192, 192, 255);
        pixmap.fill_rect(
            snap_xywh(x, y, w, h).unwrap(),
            &bg,
            Transform::identity(),
            None,
        );

        let top_left = if pressed {
            let mut p = Paint::default(); p.set_color_rgba8(128, 128, 128, 255); p
        } else {
            let mut p = Paint::default(); p.set_color_rgba8(255, 255, 255, 255); p
        };

        let bottom_right = if pressed {
            let mut p = Paint::default(); p.set_color_rgba8(255, 255, 255, 255); p
        } else {
            let mut p = Paint::default(); p.set_color_rgba8(0, 0, 0, 255); p
        };

        pixmap.fill_rect(
            snap_xywh(x, y, w, 1.0 * scale).unwrap(),
            &top_left,
            Transform::identity(),
            None,
        );
        pixmap.fill_rect(
            snap_xywh(x, y, 1.0 * scale, h).unwrap(),
            &top_left,
            Transform::identity(),
            None,
        );

        pixmap.fill_rect(
            snap_xywh(x, y + h - 1.0 * scale, w, 1.0 * scale).unwrap(),
            &bottom_right,
            Transform::identity(),
            None,
        );
        pixmap.fill_rect(
            snap_xywh(x + w - 1.0 * scale, y, 1.0 * scale, h).unwrap(),
            &bottom_right,
            Transform::identity(),
            None,
        );

        if !pressed {
            let mut dark_gray = Paint::default();
            dark_gray.set_color_rgba8(128, 128, 128, 255);
            pixmap.fill_rect(
                snap_xywh(
                    x + 1.0 * scale,
                    y + h - 2.0 * scale,
                    w - 2.0 * scale,
                    1.0 * scale,
                )
                .unwrap(),
                &dark_gray,
                Transform::identity(),
                None,
            );
            pixmap.fill_rect(
                snap_xywh(
                    x + w - 2.0 * scale,
                    y + 1.0 * scale,
                    1.0 * scale,
                    h - 2.0 * scale,
                )
                .unwrap(),
                &dark_gray,
                Transform::identity(),
                None,
            );
        }
    }

    fn draw_image(&self, pixmap: &mut Pixmap, img: &Pixmap, content_rect: Rect, fit: &Fit, rounded: bool) {
        let iw = img.width();
        let ih = img.height();
        if iw == 0 || ih == 0 {
            return;
        }

        let img_w = iw as f32;
        let img_h = ih as f32;

        let img_pixmap = img.as_ref();
        
        let path = if rounded {
            let r = (content_rect.width().min(content_rect.height()) * 0.05).max(8.0);
            rounded_rect_path(content_rect, r)
        } else {
            let mut pb = tiny_skia::PathBuilder::new();
            pb.push_rect(content_rect);
            pb.finish().unwrap()
        };
        let mut p = Paint::default();
        
        match fit {
            Fit::Stretch => {
                let sx = content_rect.width() / img_w;
                let sy = content_rect.height() / img_h;
                let transform = Transform::from_scale(sx, sy)
                    .post_translate(content_rect.x(), content_rect.y());
                p.shader = Pattern::new(
                    img_pixmap,
                    SpreadMode::Pad,
                    FilterQuality::Bilinear,
                    1.0,
                    transform,
                );
                pixmap.fill_path(&path, &p, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
            Fit::Tile => {
                let mut p = Paint::default();
                p.shader = Pattern::new(
                    img_pixmap,
                    SpreadMode::Repeat,
                    FilterQuality::Bilinear,
                    1.0,
                    Transform::from_translate(content_rect.x(), content_rect.y()),
                );
                pixmap.fill_path(&path, &p, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
            Fit::Fill => {
                let scale = (content_rect.width() / img_w).max(content_rect.height() / img_h);
                let new_w = img_w * scale;
                let new_h = img_h * scale;
                let ox = (content_rect.width() - new_w) / 2.0;
                let oy = (content_rect.height() - new_h) / 2.0;
                let x = content_rect.x() + ox;
                let y = content_rect.y() + oy;

                let transform = Transform::from_scale(scale, scale).post_translate(x, y);
                let mut p = Paint::default();
                p.shader = Pattern::new(
                    img_pixmap,
                    SpreadMode::Pad,
                    FilterQuality::Bilinear,
                    1.0,
                    transform,
                );
                pixmap.fill_path(&path, &p, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
            _ => {
                // fit
                let (scale, ox, oy) =
                    calculate_fit(img_w, img_h, content_rect.width(), content_rect.height());
                let x = content_rect.x() + ox;
                let y = content_rect.y() + oy;

                let transform = Transform::from_scale(scale, scale).post_translate(x, y);
                p.shader = Pattern::new(
                    img_pixmap,
                    SpreadMode::Pad,
                    FilterQuality::Bilinear,
                    1.0,
                    transform,
                );
                pixmap.fill_path(&path, &p, tiny_skia::FillRule::Winding, Transform::identity(), None);
            }
        }
    }

    fn draw_placeholder(&self, pixmap: &mut Pixmap, rect: Rect, msg: &str, scale: f32) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(200, 200, 200, 255);
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);

        let cx = rect.x() + rect.width() / 2.0;
        let cy = rect.y() + rect.height() / 2.0;

        self.draw_text(
            pixmap,
            "X",
            cx - 12.0 * scale,
            cy - 20.0 * scale,
            40.0 * scale,
            Color::from_rgba8(255, 0, 0, 255),
            true,
        );
        self.draw_text(
            pixmap,
            msg,
            cx - (msg.len() as f32 * 4.0 * scale),
            cy + 20.0 * scale,
            14.0 * scale,
            Color::BLACK,
            false,
        );
    }

    fn draw_text(
        &self,
        pixmap: &mut Pixmap,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        bold: bool,
    ) {
        let _ = bold; // Using one font for now
        let mut current_x = x;
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.blend_mode = BlendMode::SourceOver;

        for c in text.chars() {
            let (metrics, bitmap) = self.font.rasterize(c, size);
            if !bitmap.is_empty() {
                // Convert coverage to alpha
                let cr = color.red() * 255.0;
                let cg = color.green() * 255.0;
                let cb = color.blue() * 255.0;
                let mut rgba = Vec::with_capacity(bitmap.len() * 4);
                for &a in &bitmap {
                    let alpha_f = a as f32 / 255.0;
                    rgba.push((cr * alpha_f) as u8);
                    rgba.push((cg * alpha_f) as u8);
                    rgba.push((cb * alpha_f) as u8);
                    rgba.push(a);
                }

                if let Some(glyph_pixmap) =
                    PixmapRef::from_bytes(&rgba, metrics.width as u32, metrics.height as u32)
                {
                    pixmap.draw_pixmap(
                        0,
                        0,
                        glyph_pixmap,
                        &PixmapPaint {
                            opacity: 1.0,
                            blend_mode: BlendMode::SourceOver,
                            quality: FilterQuality::Nearest,
                        },
                        Transform::from_translate(
                            current_x + metrics.xmin as f32,
                            y + size - metrics.height as f32 - metrics.ymin as f32,
                        ),
                        None,
                    );
                }
            }
            current_x += metrics.advance_width;
        }
    }
}

// Removed close_button_rect

pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::from_rgba8(r, g, b, 255))
    } else {
        None
    }
}

pub fn calculate_fit(img_w: f32, img_h: f32, rect_w: f32, rect_h: f32) -> (f32, f32, f32) {
    let scale = (rect_w / img_w).min(rect_h / img_h);
    let new_w = img_w * scale;
    let new_h = img_h * scale;
    let ox = (rect_w - new_w) / 2.0;
    let oy = (rect_h - new_h) / 2.0;
    (scale, ox, oy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fit() {
        // Image is 100x100, rect is 200x200
        let (scale, ox, oy) = calculate_fit(100.0, 100.0, 200.0, 200.0);
        assert_eq!(scale, 2.0);
        assert_eq!(ox, 0.0);
        assert_eq!(oy, 0.0);

        // Image is 200x100, rect is 200x200
        let (scale, ox, oy) = calculate_fit(200.0, 100.0, 200.0, 200.0);
        assert_eq!(scale, 1.0);
        assert_eq!(ox, 0.0);
        assert_eq!(oy, 50.0); // (200 - 100) / 2 = 50

        // Image is 100x200, rect is 200x200
        let (scale, ox, oy) = calculate_fit(100.0, 200.0, 200.0, 200.0);
        assert_eq!(scale, 1.0);
        assert_eq!(ox, 50.0); // (200 - 100) / 2 = 50
        assert_eq!(oy, 0.0);
    }
}
fn rounded_rect_path(rect: Rect, radius: f32) -> tiny_skia::Path {
    let mut pb = tiny_skia::PathBuilder::new();
    let r = radius.min(rect.width() / 2.0).min(rect.height() / 2.0);
    let x = rect.x();
    let y = rect.y();
    let w = rect.width();
    let h = rect.height();
    
    let k = r * 0.552284749831;
    
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish().unwrap()
}
