use colored::Colorize;
use encoding_rs::Encoding;
use regex::Regex;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Instant;

use crate::extractor::Extractor;

#[derive(Debug, Clone, Copy, Default)]
pub struct ArchiveMetrics {
    pub total_bytes: u64,
    pub total_files: u32,
}

#[derive(Debug)]
pub struct ProcessOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_capture(exe: &str, args: &[&str], encoding: &str) -> anyhow::Result<ProcessOutput> {
    let mut cmd = Command::new(exe);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn()?;
    let mut stdout_pipe = child.stdout.take().unwrap();
    let mut stderr_pipe = child.stderr.take().unwrap();

    let stdout_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stdout_pipe.read_to_end(&mut buf).ok();
        buf
    });
    let stderr_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stderr_pipe.read_to_end(&mut buf).ok();
        buf
    });

    let status = child.wait()?;
    let stdout_bytes = stdout_handle.join().unwrap_or_default();
    let stderr_bytes = stderr_handle.join().unwrap_or_default();
    let encoding = parse_encoding(encoding);

    Ok(ProcessOutput {
        exit_code: status.code().unwrap_or(-1),
        stdout: decode_bytes(&stdout_bytes, encoding),
        stderr: decode_bytes(&stderr_bytes, encoding),
    })
}

fn parse_encoding(name: &str) -> Option<&'static Encoding> {
    match name.to_lowercase().as_str() {
        "utf-8" | "utf8" => Some(encoding_rs::UTF_8),
        "gbk" | "gb2312" | "gb18030" => Some(encoding_rs::GBK),
        "shift_jis" | "shift-jis" | "sjis" => Some(encoding_rs::SHIFT_JIS),
        "euc-kr" | "euckr" => Some(encoding_rs::EUC_KR),
        "big5" => Some(encoding_rs::BIG5),
        _ => None,
    }
}

fn decode_bytes(bytes: &[u8], encoding: Option<&'static Encoding>) -> String {
    if let Some(enc) = encoding {
        let (cow, _, _) = enc.decode(bytes);
        cow.into_owned()
    } else {
        String::from_utf8_lossy(bytes).to_string()
    }
}

pub fn try_extract_with_extractor(
    zip_file: &str,
    temp_folder: &str,
    password: &str,
    extractor: &dyn Extractor,
    start_time: Instant,
    ui: &Arc<crate::ui::ConsoleUi>,
    debug: bool,
    encoding: &str,
) -> anyhow::Result<bool> {
    let list_output = extractor.list(zip_file, password, encoding)?;
    if list_output.total_files == 0 && list_output.total_bytes == 0 {
        return Ok(false);
    }

    let metrics = ArchiveMetrics {
        total_bytes: list_output.total_bytes,
        total_files: list_output.total_files,
    };

    if debug {
        ui.clear_inline();
        eprintln!();
        ui.debug_section("压缩包内容");
        print_simple_file_list(&list_output.raw_stdout, ui);
    }

    if metrics.total_bytes > 0 {
        ui.info(&format!(
            "共 {} 个文件 ({})",
            metrics.total_files,
            crate::ui::format_bytes(metrics.total_bytes)
        ));
    }

    if metrics.total_bytes == 0 {
        ui.warn("无法获取文件总大小，回退到普通解压");
        extractor.extract(zip_file, temp_folder, password, encoding)?;
        return Ok(true);
    }

    run_extract_with_native_progress(
        extractor,
        zip_file,
        temp_folder,
        password,
        metrics,
        start_time,
        ui,
    )
}

#[allow(dead_code)]
pub fn try_extract(
    zip_file: &str,
    temp_folder: &str,
    password: &str,
    seven_zip_path: &str,
    start_time: Instant,
    ui: &Arc<crate::ui::ConsoleUi>,
    debug: bool,
    encoding: &str,
) -> anyhow::Result<bool> {
    let extractor = crate::extractor::sevenzip::SevenZipExtractor::new(seven_zip_path.to_string());
    try_extract_with_extractor(
        zip_file,
        temp_folder,
        password,
        &extractor,
        start_time,
        ui,
        debug,
        encoding,
    )
}

fn run_extract_with_native_progress(
    extractor: &dyn Extractor,
    zip_file: &str,
    temp_folder: &str,
    password: &str,
    metrics: ArchiveMetrics,
    start_time: Instant,
    ui: &Arc<crate::ui::ConsoleUi>,
) -> anyhow::Result<bool> {
    let exe_path = extractor.exe_path().to_string();
    let extract_args = extractor.extract_args(zip_file, temp_folder, password);
    let args_ref: Vec<&str> = extract_args.iter().map(|s| s.as_str()).collect();

    let mut cmd = Command::new(&exe_path);
    cmd.args(&args_ref)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn()?;
    let stdout_pipe = child.stdout.take().unwrap();
    let stderr_pipe = child.stderr.take().unwrap();
    let total_bytes = metrics.total_bytes;
    let ui_clone = ui.clone();

    let stdout_handle = std::thread::spawn(move || {
        track_native_progress(stdout_pipe, total_bytes, start_time, &ui_clone);
    });

    let stderr_handle = std::thread::spawn(move || {
        let mut reader = stderr_pipe;
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).ok();
        String::from_utf8_lossy(&buf).to_string()
    });

    let status = child.wait()?;
    stdout_handle.join().ok();
    let stderr_output = stderr_handle.join().unwrap_or_default();

    eprintln!();

    if status.success() {
        Ok(true)
    } else {
        let message = stderr_output.trim();
        if message.is_empty() {
            Ok(false)
        } else {
            anyhow::bail!(
                "{} 解压失败 (exit code {}): {}",
                extractor.name(),
                status.code().unwrap_or(-1),
                message
            );
        }
    }
}

fn track_native_progress(
    mut stdout_pipe: impl Read,
    total_bytes: u64,
    start_time: Instant,
    ui: &Arc<crate::ui::ConsoleUi>,
) {
    let mut buffer = [0u8; 512];
    let mut pending = String::new();
    let mut spinner_index = 0usize;
    let mut last_percent = None;

    loop {
        match stdout_pipe.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                pending.push_str(&String::from_utf8_lossy(&buffer[..n]));

                let mut segments = Vec::new();
                let mut start = 0usize;
                for (idx, ch) in pending.char_indices() {
                    if ch == '\r' || ch == '\n' {
                        segments.push(pending[start..idx].to_string());
                        start = idx + ch.len_utf8();
                    }
                }

                if start > 0 {
                    pending = pending[start..].to_string();
                }

                for segment in segments {
                    if let Some(percent) = parse_7zip_progress_percent(&segment) {
                        render_native_progress(
                            percent,
                            total_bytes,
                            start_time,
                            spinner_index,
                            ui,
                        );
                        spinner_index += 1;
                        last_percent = Some(percent);
                    }
                }
            }
            Err(_) => break,
        }
    }

    if let Some(percent) = parse_7zip_progress_percent(&pending) {
        render_native_progress(percent, total_bytes, start_time, spinner_index, ui);
        spinner_index += 1;
        last_percent = Some(percent);
    }

    if last_percent != Some(100) {
        render_native_progress(100, total_bytes, start_time, spinner_index, ui);
    }
}

fn render_native_progress(
    percent: u32,
    total_bytes: u64,
    start_time: Instant,
    spinner_index: usize,
    ui: &Arc<crate::ui::ConsoleUi>,
) {
    let percent = percent.min(100);
    let extracted_bytes = total_bytes.saturating_mul(percent as u64) / 100;
    let elapsed = start_time.elapsed().as_secs_f64();
    let extracted_mb = extracted_bytes as f64 / 1024.0 / 1024.0;
    let total_mb = total_bytes as f64 / 1024.0 / 1024.0;
    let speed_mbps = if elapsed > 0.0 {
        extracted_mb / elapsed
    } else {
        0.0
    };
    let remaining_seconds = if speed_mbps > 0.0 && percent < 100 {
        (total_mb - extracted_mb).max(0.0) / speed_mbps
    } else {
        0.0
    };

    ui.progress(
        percent,
        extracted_bytes,
        total_bytes,
        -1,
        spinner_index,
        speed_mbps,
        remaining_seconds,
    );
}

fn parse_7zip_progress_percent(text: &str) -> Option<u32> {
    static PERCENT_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = PERCENT_RE.get_or_init(|| Regex::new(r"(?m)(\d{1,3})%").unwrap());
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .map(|percent| percent.min(100))
}

fn print_simple_file_list(listing: &str, ui: &Arc<crate::ui::ConsoleUi>) {
    let mut entries: Vec<(String, bool)> = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_is_folder = false;
    let mut is_archive_header = false;

    for raw_line in listing.lines() {
        let line = raw_line.trim();

        if let Some(path_val) = line.strip_prefix("Path =") {
            let path = path_val.trim();
            if let Some(previous_path) = current_path.take() {
                if !previous_path.is_empty() && !is_archive_header {
                    entries.push((previous_path, current_is_folder));
                }
            }
            current_path = Some(path.to_string());
            current_is_folder = false;
            is_archive_header = false;
        } else if current_path.is_some() {
            if line == "Folder = +" {
                current_is_folder = true;
            } else if line.starts_with("Type =") || line.starts_with("Physical Size =") {
                is_archive_header = true;
            }
        }
    }

    if let Some(path) = current_path {
        if !path.is_empty() && !is_archive_header {
            entries.push((path, current_is_folder));
        }
    }

    if entries.is_empty() {
        return;
    }

    eprintln!();
    let max_display = 15;
    for (index, (path, is_folder)) in entries.iter().enumerate() {
        if index >= max_display {
            let remaining = entries.len() - max_display;
            ui.info(&format!("... 还有 {} 个文件/文件夹", remaining));
            break;
        }
        if *is_folder || path.ends_with('/') || path.ends_with('\\') {
            eprintln!("  {} {}", "📁".dimmed(), path.cyan());
        } else {
            eprintln!("  {} {}", "📄".dimmed(), path.white());
        }
    }
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::parse_7zip_progress_percent;

    #[test]
    fn parses_plain_percent_line() {
        assert_eq!(parse_7zip_progress_percent(" 12% - file.txt"), Some(12));
    }

    #[test]
    fn parses_percent_with_extra_text() {
        assert_eq!(
            parse_7zip_progress_percent("Extracting 100% some/path/file.txt"),
            Some(100)
        );
    }

    #[test]
    fn ignores_non_progress_text() {
        assert_eq!(parse_7zip_progress_percent("Everything is Ok"), None);
    }
}
