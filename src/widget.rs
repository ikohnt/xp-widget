use std::rc::Rc;
use winit::dpi::LogicalSize;
use winit::window::Window;
use accesskit::{Node, NodeId, Role, TreeInfo, TreeUpdate};
use accesskit_winit::Adapter;

use crate::app::clamp_to_monitor;
use crate::config::WidgetConfig;
use crate::image_src::{Directory, ImageSource, ImageState, SingleFile};
use crate::render::RenderState;

pub struct WidgetState {
    pub uid: u64,
    pub config_index: usize,
    pub config: WidgetConfig,
    pub window: Rc<Window>,
    pub surface: softbuffer::Surface<Rc<Window>, Rc<Window>>,
    pub source: Box<dyn ImageSource>,
    pub img_state: Option<ImageState>,
    pub render_state: RenderState,
    pub cursor_pos: (f32, f32),
    pub left_mouse_pressed: bool,
    pub is_focused: bool,
    pub is_hovered: bool,
    pub last_slide_time: std::time::Instant,
    pub load_generation: u64,
    pub accesskit_adapter: Option<Adapter>,
}

fn build_accessibility_tree(config: &WidgetConfig, current_path: Option<&str>) -> TreeUpdate {
    let window_id = NodeId(1);
    let image_id = NodeId(2);

    let mut window = Node::new(Role::Window);
    window.set_label(config.window_title.clone());
    window.set_children(vec![image_id]);

    let mut image = Node::new(Role::Image);
    let alt = if !config.alt_text.is_empty() {
        config.alt_text.clone()
    } else {
        let path_to_use = current_path.unwrap_or(&config.image_path);
        std::path::Path::new(path_to_use)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Image".to_string())
    };
    image.set_label(alt);

    TreeUpdate {
        nodes: vec![
            (window_id, window),
            (image_id, image),
        ],
        tree: Some(TreeInfo::new(window_id)),
        focus: window_id,
        tree_id: accesskit::TreeId::ROOT,
    }
}




impl WidgetState {
    pub fn new(
        uid: u64,
        config_index: usize,
        config: WidgetConfig,
        window: Rc<Window>,
        surface: softbuffer::Surface<Rc<Window>, Rc<Window>>,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Self {
        let path = std::path::Path::new(&config.image_path);
        let source: Box<dyn ImageSource> = if path.is_dir() {
            Box::new(Directory::new(config.image_path.clone()))
        } else {
            Box::new(SingleFile {
                path: config.image_path.clone(),
            })
        };

        let proxy = crate::events::get_proxy().expect("Event proxy not initialized");
        let adapter = accesskit_winit::Adapter::with_event_loop_proxy(event_loop, &window, proxy);

        let mut state = Self {
            uid,
            config_index,
            config,
            window,
            surface,
            source,
            img_state: None,
            render_state: RenderState::default(),
            cursor_pos: (0.0, 0.0),
            left_mouse_pressed: false,
            is_focused: true,
            is_hovered: false,
            last_slide_time: std::time::Instant::now(),
            load_generation: 0,
            accesskit_adapter: Some(adapter),
        };
        state.update_accessibility_tree();
        state.begin_load();
        state
    }

    pub fn update_accessibility_tree(&mut self) {
        if let Some(adapter) = &mut self.accesskit_adapter {
            let path = self.source.current_path();
            let tree = build_accessibility_tree(&self.config, path.as_deref());
            adapter.update_if_active(|| tree);
        }
    }

    pub fn begin_load(&mut self) {
        self.load_generation += 1;
        self.img_state = Some(ImageState::Loading);
        self.window.request_redraw();

        let uid = self.uid;
        let gen = self.load_generation;
        let config = self.config.clone();
        
        let path = self.source.current_path().unwrap_or_default();
        
        if path.is_empty() {
            crate::events::send(crate::events::AppEvent::ImageDecoded {
                uid,
                generation: gen,
                result: Err("No image path".to_string()),
            });
            return;
        }

        std::thread::spawn(move || {
            let res = crate::image_src::decode_image(&path, &config);
            crate::events::send(crate::events::AppEvent::ImageDecoded {
                uid,
                generation: gen,
                result: res,
            });
        });
    }

    #[allow(dead_code)]
    pub fn reload_image(&mut self) {
        let path = std::path::Path::new(&self.config.image_path);
        self.source = if path.is_dir() {
            Box::new(Directory::new(self.config.image_path.clone()))
        } else {
            Box::new(SingleFile {
                path: self.config.image_path.clone(),
            })
        };
        self.begin_load();

        let (logical_w, logical_h) = self.config.window_logical_size();
        let _ = self
            .window
            .request_inner_size(LogicalSize::new(logical_w, logical_h));

        clamp_to_monitor(&self.window);
        self.window.request_redraw();
    }
}
