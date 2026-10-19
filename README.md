# XP Widget

A lightweight, Windows 10/11 exclusive desktop image widget written in Rust. It allows you to place multiple customized image widgets on your screen with extremely low memory footprint. Rendered entirely on the CPU using `tiny-skia`, these widgets idle at near 0% CPU footprint.

## Features
- **Aesthetic Themes:** Choose between classic Windows 95, Windows 98, Windows XP (Luna), Windows Vista/7 (Aero Glass with real DWM blur), and MacOS Aqua!
- **Settings UI:** A microscopic Native Windows GUI (`nwg`) accessible right from your system tray to tweak opacity, scale, dimension, theme, and ghost mode visually. No manual JSON/TOML editing required!
- **Slideshows & GIFs:** Point a widget to a folder of images to automatically cycle through them based on an interval, or point it to an animated `.gif`!
- **Standard Layer:** Draggable, windowed frame that overlays your applications. Includes a custom right-click context menu.
- **Desktop Layer:** Sandwiched behind your desktop icons (`WorkerW` technique), acting as a customized desktop accessory. Completely ignores mouse input and stays hidden from the Taskbar / Alt-Tab.
- **Ghost Mode:** Make any floating widget completely click-through, allowing you to click the windows behind it!
- **IPC Support:** CLI controls allow you to reload, add widgets, open settings, or quit the widget programmatically.

## Building
This project uses Rust 2021 edition and MSVC.
1. Install Rust (`rustup default stable-x86_64-pc-windows-msvc`).
2. Clone this repository.
3. Build the release binary:
```powershell
cargo build --release
```

## Configuration & Usage
Upon launching `xp-widget.exe`, a classic Windows XP tray icon will appear in your system tray. 
Right-click the tray icon to:
- **Settings:** Open the Settings UI to adjust widget properties.
- **Add Widget:** Spawns a new widget. 
- **Start with Windows:** Uses the `auto-launch` crate to automatically register the widget to start silently when you log in.

### The Settings UI
The Settings UI allows you to quickly tweak:
- **Theme:** Classic (Light Gray), Win98 (Gradient), Aero Glass (DWM Blur), MacOS Aqua.
- **Opacity (0-1):** Fades the image and window.
- **Scale (x):** Uniformly resizes the entire widget.
- **Width / Height (px):** The exact logical dimensions of the image bounding box.
- **Slideshow Interval (s):** Seconds to wait before cycling to the next image in a directory.
- **Ghost Mode (Click-through):** Applies the `WS_EX_TRANSPARENT` flag so your mouse clicks pass through the widget to the window behind it.

### Advanced Configuration (`config.toml`)
If you want granular control, you can click **Edit Config** in the tray menu. The file lives at `%APPDATA%\xp-widget\config.toml`.
- `image_path` (string): Absolute path to an image file or a directory.
- `layer` (string): `"standard"` (floating window) or `"desktop"` (behind desktop icons).
- `chrome` (boolean): If `true`, renders the OS title bar.
- `frame` (boolean): If `true`, renders a white polaroid frame with a drop shadow underneath the image.
- `fit` (string): `"fit"`, `"stretch"`, `"tile"`, `"cover"`, `"scale-down"`, `"none"`.
- `rotation` (integer): Image rotation (0, 90, 180, 270).

## CLI Flags
If the widget is running as a background task, you can control it via CLI:
- `xp-widget.exe --reload` : Signals the running instance to hot-reload `config.toml`.
- `xp-widget.exe --show` : Opens the Settings UI.
- `xp-widget.exe --add` : Spawns a new widget.
- `xp-widget.exe --quit` : Safely signals the widget to exit.

## Licenses and Assets
- The Open Sans font used in the title bar is embedded via `fontdue` and licensed under the SIL Open Font License (OFL).
- No proprietary OS assets are included; all visuals (borders, drop shadows, gradients, Aqua buttons) are meticulously synthesized through vector calculations in code.
- The tray icon is an original pixel-art embedded JPEG.

## Known Limitations
- The desktop layer (`layer = "desktop"`) acts as a display-only surface. Mouse inputs always fall through to the real desktop beneath it.
- The `WorkerW` progression technique is an undocumented Win32 shell feature and may exhibit different layering behaviors on future Insider builds of Windows 11.
- Due to the nature of Windows desktop hooking, switching primary monitors abruptly may cause temporary visual stutters before the widget snaps to position.
