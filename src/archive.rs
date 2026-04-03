use colored::Colorize;
use regex::Regex;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Default)]
pub struct ArchiveMetrics {
    pub total_bytes: u64,
    pub total_files: u32,
}

#[derive(Debug)]
pub struct ProcessOutput {
    pub exit_code: i32,
    pub stdout: String,
    #[allow(dead_code)]
    pub stderr: String,
}

pub fn run_capture(exe: &str, args: &[&str]) -> anyhow::Result<ProcessOutput> {
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

    let t1 = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stdout_pipe.read_to_end(&mut buf).ok();
        buf
    });
    let t2 = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stderr_pipe.read_to_end(&mut buf).ok();
        buf
    });

    let status = child.wait()?;
    let stdout_bytes = t1.join().unwrap_or_default();
    let stderr_bytes = t2.join().unwrap_or_default();

    Ok(ProcessOutput {
        exit_code: status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
        stderr: String::from_utf8_lossy(&stderr_bytes).to_string(),
    })
}

pub fn try_extract(
    zip_file: &str,
    temp_folder: &str,
    password: &str,
    seven_zip_path: &str,
    start_time: Instant,
    ui: &Arc<crate::ui::ConsoleUi>,
    debug: bool,
) -> anyhow::Result<bool> {
    let pwd_flag = format!("-p:{password}");
    let list_args = vec!["l", "-list:v", "-y", &pwd_flag, zip_file];

    let list_result = run_capture(seven_zip_path, &list_args)?;

    if list_result.exit_code != 0 {
        return Ok(false);
    }

    if debug {
        let simple_list_args = vec!["l", "-list:s", "-y", &pwd_flag, zip_file];
        if let Ok(simple_result) = run_capture(seven_zip_path, &simple_list_args) {
            ui.clear_inline();
            eprintln!();
            ui.debug_section("压缩包内容");
            print_simple_file_list(&simple_result.stdout, ui);
        }
    }

    if let Some(metrics) = parse_listing_metrics(&list_result.stdout) {
        ui.info(&format!(
            "共 {} 个文件 ({})",
            metrics.total_files,
            crate::ui::format_bytes(metrics.total_bytes)
        ));

        let out_flag = format!("-o:{temp_folder}");
        let extract_args = vec!["x", "-y", &pwd_flag, &out_flag, zip_file];

        run_extract_with_progress(
            seven_zip_path,
            &extract_args,
            temp_folder,
            metrics,
            start_time,
            ui,
        )
    } else {
        ui.warn("无法获取文件大小，进度不可用");

        let out_flag = format!("-o:{temp_folder}");
        let extract_args = vec!["x", "-y", &pwd_flag, &out_flag, zip_file];

        run_extract_simple(seven_zip_path, &extract_args, ui)
    }
}

pub fn parse_listing_metrics(listing: &str) -> Option<ArchiveMetrics> {
    let re_with_date = Regex::new(
        r"^(\d{4}[-/]\d{2}[-/]\d{2})\s+(\d{2}:\d{2}:\d{2})\s+(\S+)\s+(\d+)\s+(\d+)\s+(.+)$",
    )
    .ok()?;
    let re_no_date = Regex::new(r"^(\S+)\s+(\d+)\s+(\d+)\s+(.+)$").ok()?;

    let mut total_bytes: u64 = 0;
    let mut files: u32 = 0;

    for raw in listing.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("----")
            || line.starts_with("Path =")
            || line.starts_with("Type =")
            || line.starts_with("Physical Size =")
            || line.starts_with("Headers Size =")
            || line.starts_with("Solid =")
            || line.starts_with("Blocks =")
            || line.starts_with("Files:")
            || line.starts_with("Folders:")
        {
            continue;
        }
        if line.contains("Name") && line.contains("Size") && line.to_lowercase().contains("comp") {
            continue;
        }
        if line.contains("files,") {
            continue;
        }

        let size_str: Option<&str> = if let Some(caps) = re_with_date.captures(line) {
            caps.get(4).map(|m| m.as_str())
        } else if let Some(caps) = re_no_date.captures(line) {
            caps.get(2).map(|m| m.as_str())
        } else {
            None
        };

        if let Some(s) = size_str {
            if let Ok(size) = s.parse::<u64>() {
                total_bytes += size;
                files += 1;
            }
        }
    }

    if total_bytes > 0 && files > 0 {
        Some(ArchiveMetrics {
            total_bytes,
            total_files: files,
        })
    } else {
        None
    }
}

fn run_extract_with_progress(
    seven_zip_path: &str,
    args: &[&str],
    temp_folder: &str,
    metrics: ArchiveMetrics,
    start_time: Instant,
    ui: &Arc<crate::ui::ConsoleUi>,
) -> anyhow::Result<bool> {
    let mut cmd = Command::new(seven_zip_path);
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
    });

    let stderr_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stderr_pipe.read_to_end(&mut buf).ok();
    });

    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();
    let temp_folder_owned = temp_folder.to_string();
    let total_bytes = metrics.total_bytes;
    let ui_clone = ui.clone();

    let progress_handle = std::thread::spawn(move || {
        track_progress(
            &temp_folder_owned,
            total_bytes,
            start_time,
            &done_clone,
            &ui_clone,
        );
    });

    let status = child.wait()?;
    done.store(true, Ordering::Release);
    stdout_handle.join().ok();
    stderr_handle.join().ok();
    progress_handle.join().ok();

    eprintln!();

    Ok(status.success())
}

fn run_extract_simple(
    seven_zip_path: &str,
    args: &[&str],
    ui: &Arc<crate::ui::ConsoleUi>,
) -> anyhow::Result<bool> {
    let mut cmd = Command::new(seven_zip_path);
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

    let t1 = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stdout_pipe.read_to_end(&mut buf).ok();
    });
    let t2 = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stderr_pipe.read_to_end(&mut buf).ok();
    });

    let mut spinner_index = 0usize;
    while child.try_wait()?.is_none() {
        ui.progress_unknown(spinner_index);
        spinner_index += 1;
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    t1.join().ok();
    t2.join().ok();

    let status = child.wait()?;
    eprintln!();

    Ok(status.success())
}

fn track_progress(
    temp_folder: &str,
    total_bytes: u64,
    start_time: Instant,
    done: &AtomicBool,
    ui: &Arc<crate::ui::ConsoleUi>,
) {
    let mut last_percent: i32 = -1;
    let mut last_bytes: u64 = u64::MAX;
    let mut spinner_index = 0usize;
    let mut last_write = Instant::now();

    while !done.load(Ordering::Acquire) {
        let (extracted_bytes, file_count) = measure_folder(temp_folder).unwrap_or((0, 0));

        let elapsed = start_time.elapsed().as_secs_f64();
        let extracted_mb = extracted_bytes as f64 / 1024.0 / 1024.0;
        let total_mb = total_bytes as f64 / 1024.0 / 1024.0;
        let speed_mbps = if elapsed > 0.0 {
            extracted_mb / elapsed
        } else {
            0.0
        };
        let remaining_seconds = if speed_mbps > 0.0 {
            (total_mb - extracted_mb) / speed_mbps
        } else {
            0.0
        };

        let percent = if total_bytes > 0 {
            ((extracted_bytes as f64 * 100.0) / total_bytes as f64).clamp(0.0, 100.0) as i32
        } else {
            0
        };

        let should_write = percent != last_percent
            || extracted_bytes != last_bytes
            || last_write.elapsed().as_millis() > 500;

        if should_write {
            last_percent = percent;
            last_bytes = extracted_bytes;
            ui.progress(
                percent as u32,
                extracted_bytes,
                total_bytes,
                file_count as i32,
                spinner_index,
                speed_mbps,
                remaining_seconds,
            );
            spinner_index += 1;
            last_write = Instant::now();
        }

        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    // Final 100% progress
    ui.progress(100, total_bytes, total_bytes, -1, spinner_index, 0.0, 0.0);
}

fn measure_folder(folder: &str) -> std::io::Result<(u64, u32)> {
    let mut total_bytes = 0u64;
    let mut file_count = 0u32;

    if !std::path::Path::new(folder).exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "folder not found",
        ));
    }

    fn walk(dir: &std::path::Path, bytes: &mut u64, count: &mut u32) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, bytes, count);
            } else if let Ok(meta) = std::fs::metadata(&path) {
                *bytes += meta.len();
                *count += 1;
            }
        }
    }

    walk(
        std::path::Path::new(folder),
        &mut total_bytes,
        &mut file_count,
    );
    Ok((total_bytes, file_count))
}

fn print_simple_file_list(listing: &str, ui: &Arc<crate::ui::ConsoleUi>) {
    let lines: Vec<&str> = listing
        .lines()
        .map(|l| l.trim())
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("----")
                && !line.starts_with("bz ")
                && !line.contains("Bandizip")
                && !line.contains("Copyright")
                && !line.starts_with("Listing archive:")
                && !line.starts_with("Archive format:")
                && !line.contains("files,")
        })
        .collect();

    if lines.is_empty() {
        return;
    }

    eprintln!();
    let max_display = 15;
    for (i, line) in lines.iter().enumerate() {
        if i >= max_display {
            let remaining = lines.len() - max_display;
            ui.info(&format!("... 还有 {} 个文件/文件夹", remaining));
            break;
        }
        if line.ends_with('/') || line.ends_with('\\') {
            eprintln!("  {} {}", "📁".dimmed(), line.cyan());
        } else {
            eprintln!("  {} {}", "📄".dimmed(), line.white());
        }
    }
    eprintln!();
}
