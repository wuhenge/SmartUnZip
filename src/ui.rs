use colored::Colorize;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

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
        let line = "━".repeat(width);
        eprintln!("{}", line.dimmed());
        eprintln!("  {}  {}", self.app_name.cyan().bold(), title.cyan());
        eprintln!("{}", line.dimmed());
    }

    pub fn info(&self, msg: &str) {
        self.clear_inline();
        eprintln!("  {} {}", "●".dimmed(), msg.dimmed());
    }

    pub fn success(&self, msg: &str) {
        self.clear_inline();
        eprintln!("  {} {}", "✓".green().bold(), msg.green());
    }

    pub fn warn(&self, msg: &str) {
        self.clear_inline();
        eprintln!("  {} {}", "⚠".yellow().bold(), msg.yellow());
    }

    pub fn error(&self, msg: &str) {
        self.clear_inline();
        eprintln!("  {} {}", "✗".red().bold(), msg.red());
    }

    pub fn attempt_password(
        &self,
        index: usize,
        total: usize,
        password: &str,
        debug_cmd: Option<&str>,
    ) {
        let spinner_idx = self.attempt_spinner.fetch_add(1, Ordering::Relaxed);
        let spinner = SPINNER_FRAMES[spinner_idx % SPINNER_FRAMES.len()];
        let masked = mask_password(password);
        
        let progress = format!("[{}/{}]", index, total);
        if let Some(cmd) = debug_cmd {
            self.inline(&format!(
                "\r  {} {} 尝试密码: {} │ {}",
                spinner.cyan(),
                progress.dimmed(),
                masked.yellow(),
                cmd.dimmed()
            ));
        } else {
            self.inline(&format!(
                "\r  {} {} 尝试密码: {}",
                spinner.cyan(),
                progress.dimmed(),
                masked.yellow()
            ));
        }
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
        const BAR_WIDTH: usize = 24;
        
        let spinner = SPINNER_FRAMES[spinner_index % SPINNER_FRAMES.len()];
        let bar = build_progress_bar(percent as usize, BAR_WIDTH);
        
        let file_part = if files >= 0 {
            format!(" {} 文件", files)
        } else {
            String::new()
        };

        let speed_str = if speed_mbps > 0.0 {
            format!("{:.1} MB/s", speed_mbps)
        } else {
            String::new()
        };
        
        let time_str = format_time(remaining_seconds);
        let speed_and_time = match (!speed_str.is_empty(), !time_str.is_empty()) {
            (true, true) => format!(" {} │ {}", speed_str.cyan(), time_str.white()),
            (true, false) => format!(" {}", speed_str.cyan()),
            (false, true) => format!(" {}", time_str.white()),
            _ => String::new(),
        };

        let percent_str = format!("{:>3}%", percent);
        let percent_colored = if percent < 30 {
            percent_str.yellow()
        } else if percent < 70 {
            percent_str.cyan()
        } else {
            percent_str.green()
        };

        let msg = format!(
            "\r  {} {} {} {}{}{}",
            spinner.cyan(),
            percent_colored.bold(),
            bar,
            format_bytes(extracted_bytes).white(),
            file_part.dimmed(),
            speed_and_time
        );
        self.inline(&msg);
    }

    pub fn progress_unknown(&self, spinner_index: usize) {
        let spinner = SPINNER_FRAMES[spinner_index % SPINNER_FRAMES.len()];
        let dots = ".".repeat((spinner_index % 3) + 1);
        self.inline(&format!("\r  {} 解压中{}", spinner.cyan(), dots.white()));
    }

    fn inline(&self, message: &str) {
        let mut last = self.last_inline_len.lock().unwrap();
        let display_width = visible_width(message);
        let pad = last.saturating_sub(display_width);
        let padded = format!("{}{}", message, " ".repeat(pad));
        eprint!("{padded}");
        std::io::stderr().flush().ok();
        *last = display_width;
    }

    fn clear_inline(&self) {
        let mut last = self.last_inline_len.lock().unwrap();
        if *last > 0 {
            eprintln!();
            *last = 0;
        }
    }
}

fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if in_escape {
            if chars[i] == 'm' {
                in_escape = false;
            }
            i += 1;
            continue;
        }
        
        if chars[i] == '\x1b' && i + 1 < chars.len() && chars[i + 1] == '[' {
            in_escape = true;
            i += 2;
            continue;
        }
        
        width += unicode_width(chars[i]);
        i += 1;
    }
    
    width
}

fn unicode_width(c: char) -> usize {
    match c {
        '\x00'..='\x7F' => 1,
        _ => 2,
    }
}

fn build_progress_bar(percent: usize, width: usize) -> String {
    let filled = (width as f64 * percent as f64 / 100.0).round() as usize;
    let filled = filled.min(width);
    
    let mut bar = String::new();
    
    for i in 0..width {
        if i < filled {
            bar.push_str(&"█".cyan().to_string());
        } else {
            bar.push_str(&"░".dimmed().to_string());
        }
    }
    
    format!("│{}│", bar)
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
        format!("剩余 {:.0}s", seconds)
    } else if seconds < 3600.0 {
        let mins = (seconds / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        format!("剩余 {}m{:02}s", mins, secs)
    } else {
        let hours = (seconds / 3600.0) as u32;
        let mins = ((seconds % 3600.0) / 60.0) as u32;
        format!("剩余 {}h{:02}m", hours, mins)
    }
}

fn terminal_width() -> usize {
    80
}
