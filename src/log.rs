use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static VERBOSE: AtomicBool = AtomicBool::new(false);
static LOG_LOCK: Mutex<()> = Mutex::new(());
const MAX_LOG_BYTES: u64 = 1024 * 1024;

pub fn log_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut p| {
        p.push("xp-widget");
        p.push("widget.log");
        p
    })
}

/// Call once at startup. Rotates the log so it can't grow forever.
pub fn init(verbose: bool) {
    set_verbose(verbose);
    if let Some(path) = log_path() {
        if std::fs::metadata(&path)
            .map(|m| m.len() > MAX_LOG_BYTES)
            .unwrap_or(false)
        {
            let _ = std::fs::rename(&path, path.with_extension("old.log"));
        }
    }
}

pub fn set_verbose(enabled: bool) {
    VERBOSE.store(enabled, Ordering::Relaxed);
}

/// UTC timestamp "YYYY-MM-DD HH:MM:SS" without pulling in a date crate.
fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Civil-from-days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn append_log(level: &str, msg: &str) {
    let _guard = LOG_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let path = std::env::var("XP_WIDGET_LOG_PATH")
        .map(PathBuf::from)
        .ok()
        .or_else(log_path);

    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{} [{}] {}", timestamp(), level, msg);
        }
    }
}

pub fn error(msg: &str) {
    append_log("ERROR", msg);
}

pub fn info(msg: &str) {
    append_log("INFO", msg);
}

pub fn verbose(msg: &str) {
    if VERBOSE.load(Ordering::Relaxed) {
        append_log("VERBOSE", msg);
    }
}
