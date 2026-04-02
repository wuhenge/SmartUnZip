use colored::Colorize;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub struct ConsoleUi {
    app_name: String,
    last_inline_len: Mutex<usize>,
    attempt_spinner: AtomicUsize,
}

impl ConsoleUi {
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            last_inline_len: Mutex::new(0),
            attempt_spinner: AtomicUsize::new(0),
        }
    }

    pub fn header(&self, title: &str) {
        let width = std::cmp::min(std::cmp::max(terminal_width().saturating_sub(1), 20), 80);
        let line = "=".repeat(width);
        eprintln!("{}", line.dimmed());
        eprintln!("  {}  {}", self.app_name.cyan().bold(), title.cyan());
        eprintln!("{}", line.dimmed());
    }

    pub fn info(&self, msg: &str) {
        self.clear_inline();
        eprintln!("  {} {}", "[*]".dimmed(), msg.dimmed());
    }

    pub fn success(&self, msg: &str) {
        self.clear_inline();
        eprintln!("  {} {}", "[+]".green().bold(), msg.green());
    }

    pub fn warn(&self, msg: &str) {
        self.clear_inline();
        eprintln!("  {} {}", "[!]".yellow().bold(), msg.yellow());
    }

    pub fn error(&self, msg: &str) {
        self.clear_inline();
        eprintln!("  {} {}", "[-]".red().bold(), msg.red());
    }

    pub fn attempt_password(&self, index: usize, total: usize, password: &str) {
        let frames = ["..", ".+", "+.", "+-"];
        let spinner_idx = self.attempt_spinner.fetch_add(1, Ordering::Relaxed);
        let spinner_str = frames[spinner_idx % frames.len()];
        let masked = mask_password(password);
        self.inline(&format!(
            "\r  {spinner_str} [{index}/{total}] 密码: {masked}"
        ));
    }

    pub fn progress(
        &self,
        percent: u32,
        extracted_bytes: u64,
        _total_bytes: u64,
        files: i32,
        spinner_index: usize,
        speed_mbps: f64,
        remaining_seconds: f64,
    ) {
        let percent = percent.clamp(0, 100);
        const BAR_WIDTH: usize = 20;
        let filled = ((BAR_WIDTH as f64) * (percent as f64 / 100.0)).round() as usize;
        let filled = filled.clamp(0, BAR_WIDTH);

        let frames = ["..", ".+", "+.", "+-"];
        let spinner = frames[spinner_index % frames.len()];

        let bar_filled = "#".repeat(filled);
        let bar_empty = ".".repeat(BAR_WIDTH - filled);

        let file_part = if files >= 0 {
            format!(" {files} 个文件")
        } else {
            String::new()
        };

        let speed_str = if speed_mbps > 0.0 {
            format!("{speed_mbps:.1} MB/s")
        } else {
            String::new()
        };
        let time_str = format_time(remaining_seconds);
        let speed_and_time = match (!speed_str.is_empty(), !time_str.is_empty()) {
            (true, true) => format!(" {speed_str} | {time_str}"),
            (true, false) => format!(" {speed_str}"),
            (false, true) => format!(" {time_str}"),
            _ => String::new(),
        };

        let msg = format!(
            "\r  {spinner} {percent:>3}% [{bar_filled}{bar_empty}] {}{file_part}{speed_and_time}",
            format_bytes(extracted_bytes),
        );
        self.inline(&msg);
    }

    pub fn progress_unknown(&self, spinner_index: usize) {
        let frames = ["..", ".+", "+.", "+-"];
        let spinner = frames[spinner_index % frames.len()];
        self.inline(&format!("\r  {spinner} 解压中..."));
    }

    fn inline(&self, message: &str) {
        let mut last = self.last_inline_len.lock().unwrap();
        let pad = last.saturating_sub(message.len());
        let padded = format!("{}{}", message, " ".repeat(pad));
        eprint!("{padded}");
        std::io::stderr().flush().ok();
        *last = message.len();
    }

    fn clear_inline(&self) {
        let mut last = self.last_inline_len.lock().unwrap();
        if *last > 0 {
            eprintln!();
            *last = 0;
        }
    }
}

fn mask_password(password: &str) -> String {
    if password.is_empty() {
        return "(空)".to_string();
    }
    let len = password.chars().count();
    if len <= 2 {
        return "*".repeat(len);
    }
    if len <= 6 {
        let first = password.chars().next().unwrap();
        let last = password.chars().last().unwrap();
        return format!("{first}{}{last}", "*".repeat(len - 2));
    }
    let first_two: String = password.chars().take(2).collect();
    let last_two: String = password.chars().skip(len - 2).collect();
    format!("{first_two}{}{last_two}", "*".repeat(len - 4))
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

fn format_time(seconds: f64) -> String {
    if seconds <= 0.0 || seconds.is_infinite() {
        return String::new();
    }
    if seconds < 60.0 {
        format!("剩余 {seconds:.0}s")
    } else if seconds < 3600.0 {
        let mins = (seconds / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        format!("剩余 {mins}m{secs:02}s")
    } else {
        let hours = (seconds / 3600.0) as u32;
        let mins = ((seconds % 3600.0) / 60.0) as u32;
        format!("剩余 {hours}h{mins:02}m")
    }
}

fn terminal_width() -> usize {
    80
}
