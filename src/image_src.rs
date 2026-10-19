use crate::config::WidgetConfig;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use std::path::{Path, PathBuf};

use tiny_skia::Pixmap;

pub enum LoadedImage {
    Static(Pixmap),
    Animated {
        frames: Vec<Pixmap>,
        delays: Vec<u32>,
    },
}

pub enum ImageState {
    Loading,
    Static(Pixmap),
    Animated {
        frames: Vec<Pixmap>,
        delays: Vec<u32>,
        current_frame: usize,
        last_frame_time: std::time::Instant,
    },
    Error(String),
}

pub trait ImageSource {
    fn current_path(&self) -> Option<String>;
    fn next(&mut self) {}
    fn prev(&mut self) {}
    fn random(&mut self) {}
}

pub struct SingleFile {
    pub path: String,
}

impl ImageSource for SingleFile {
    fn current_path(&self) -> Option<String> {
        if self.path.is_empty() {
            None
        } else {
            Some(self.path.clone())
        }
    }
}

pub struct Directory {
    pub _path: String,
    pub files: Vec<PathBuf>,
    pub current_index: usize,
}

impl Directory {
    pub fn new(path: String) -> Self {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let path = entry.path();
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            let ext = ext.to_lowercase();
                            if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "gif" {
                                files.push(path);
                            }
                        }
                    }
                }
            }
        }
        files.sort();
        Self {
            _path: path,
            files,
            current_index: 0,
        }
    }
}

impl ImageSource for Directory {
    fn current_path(&self) -> Option<String> {
        if self.files.is_empty() {
            None
        } else {
            Some(self.files[self.current_index].to_string_lossy().to_string())
        }
    }

    fn next(&mut self) {
        if !self.files.is_empty() {
            self.current_index = (self.current_index + 1) % self.files.len();
        }
    }

    fn prev(&mut self) {
        if !self.files.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.files.len() - 1;
            } else {
                self.current_index -= 1;
            }
        }
    }

    fn random(&mut self) {
        if !self.files.is_empty() {
            if let Ok(dur) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                // A very simple pseudo-random based on nanoseconds
                let seed = dur.as_nanos() as usize;
                // Add a small prime to ensure it doesn't stay the same if called rapidly
                self.current_index = (seed.wrapping_mul(17) + 11) % self.files.len();
            }
        }
    }
}

pub fn decode_image(path_str: &str, config: &WidgetConfig) -> Result<LoadedImage, String> {
    if path_str.is_empty() {
        return Err("No image path".to_string());
    }

    let path = Path::new(path_str);
    if !path.exists() {
        return Err("File not found".to_string());
    }

    let is_gif = path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("gif"));
    if is_gif {
        if let Ok(file) = std::fs::File::open(path) {
            if let Ok(decoder) = image::codecs::gif::GifDecoder::new(file) {
                use image::AnimationDecoder;
                if let Ok(frames) = decoder.into_frames().collect_frames() {
                    if frames.len() > 1 {
                        let mut processed_frames = Vec::new();
                        let mut delays = Vec::new();
                        for frame in frames {
                            let (num, denom) = frame.delay().numer_denom_ms();
                            let delay_ms = if denom == 0 { 100 } else { (num as f32 / denom as f32) as u32 };
                            let img = DynamicImage::ImageRgba8(frame.into_buffer());
                            processed_frames.push(process_image(img, config));
                            delays.push(delay_ms.max(20)); // Minimum 20ms delay
                        }
                        return Ok(LoadedImage::Animated {
                            frames: processed_frames,
                            delays,
                        });
                    }
                }
            }
        }
    }

    match image::open(path) {
        Ok(img) => {
            Ok(LoadedImage::Static(process_image(img, config)))
        }
        Err(e) => {
            Err(format!("Decode failed: {}", e))
        }
    }
}

fn process_image(mut img: DynamicImage, config: &WidgetConfig) -> Pixmap {
    img = match config.rotation {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => img,
    };

    // Cap image dimensions to save massive amounts of memory
    // (a 4K image uses 33MB of RAM, a 1200x1200 uses ~5.7MB)
    let max_dim = 1200;
    let (orig_w, orig_h) = img.dimensions();
    if orig_w > max_dim || orig_h > max_dim {
        img = img.resize(max_dim, max_dim, image::imageops::FilterType::Triangle);
    }

    let bg_color = parse_hex_color(&config.background_color).unwrap_or([0, 0, 0, 255]);
    let bg_r = bg_color[0] as f32;
    let bg_g = bg_color[1] as f32;
    let bg_b = bg_color[2] as f32;

    let b_factor = config.brightness;
    let o_factor = config.opacity;

    let (w, h) = img.dimensions();
    let mut new_img = ImageBuffer::new(w, h);

    for (x, y, pixel) in img.pixels() {
        let r = pixel[0] as f32 * b_factor;
        let g = pixel[1] as f32 * b_factor;
        let b = pixel[2] as f32 * b_factor;
        let a = (pixel[3] as f32 / 255.0) * o_factor;

        let final_r = (r * a + bg_r * (1.0 - a)).clamp(0.0, 255.0) as u8;
        let final_g = (g * a + bg_g * (1.0 - a)).clamp(0.0, 255.0) as u8;
        let final_b = (b * a + bg_b * (1.0 - a)).clamp(0.0, 255.0) as u8;

        new_img.put_pixel(x, y, Rgba([final_r, final_g, final_b, 255]));
    }

    let rgba = new_img.into_raw();
    let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();
    pixmap.data_mut().copy_from_slice(&rgba);
    pixmap
}

pub fn parse_hex_color(hex: &str) -> Option<[u8; 4]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 || hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()?
        } else {
            255
        };
        Some([r, g, b, a])
    } else {
        None
    }
}
