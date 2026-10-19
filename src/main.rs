#![windows_subsystem = "windows"]

use winit::event_loop::{ControlFlow, EventLoop};

pub mod app;
pub mod config;
pub mod events;
pub mod image_src;
pub mod ipc;
pub mod layer;
pub mod log;
pub mod render;
pub mod tray;
pub mod watcher;
pub mod widget;
pub mod settings_ui;

use app::App;
use events::AppEvent;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--reload") {
        ipc::send_reload();
        return;
    }
    if args.iter().any(|a| a == "--quit") {
        ipc::send_quit();
        return;
    }
    if args.iter().any(|a| a == "--show") || args.iter().any(|a| a == "--settings") {
        ipc::send_show();
        return;
    }
    if args.iter().any(|a| a == "--add") {
        ipc::send_add();
        return;
    }

    let loaded = config::load_or_create();
    log::init(loaded.config.verbose);
    log::info("Starting xp-widget");
    log::verbose("Verbose logging enabled");

    let Some(ipc_server) = ipc::IpcServer::new() else {
        if args.len() == 1 {
            ipc::send_add();
        }
        log::error("xp-widget is already running. Exiting.");
        return;
    };

    let event_loop = EventLoop::<AppEvent>::with_user_event().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let proxy = event_loop.create_proxy();
    events::init(proxy);

    ipc_server.start_listener();

    if let Some(path) = config::AppConfig::path() {
        watcher::start(path);
    }

    let mut app = App::new(loaded.config);
    let _ = event_loop.run_app(&mut app);
}
