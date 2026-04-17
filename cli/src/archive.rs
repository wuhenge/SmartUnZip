use colored::Colorize;
use encoding_rs::Encoding;
use regex::Regex;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
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
    #[allow(dead_code)]
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

    let enc = parse_encoding(encoding);

    Ok(ProcessOutput {
        exit_code: status.code().unwrap_or(-1),
        stdout: decode_bytes(&stdout_bytes, enc),
        stderr: decode_bytes(&stderr_bytes, enc),
    })
}

/// 根据编码名称解析 encoding_rs Encoding
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

/// 用指定编码解码字节流，回退到 UTF-8 lossy
fn decode_bytes(bytes: &[u8], encoding: Option<&'static Encoding>) -> String {
    if let Some(enc) = encoding {
        let (cow, _encoding_used, _had_errors) = enc.decode(bytes);
        cow.into_owned()
    } else {
        String::from_utf8_lossy(bytes).to_string()
    }
}

/// 使用 Extractor trait 尝试解压（新接口）
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
    // 1. 列出文件
    let list_output = extractor.list(zip_file, password, encoding)?;

    if list_output.total_files == 0 && list_output.total_bytes == 0 {
        // 可能密码错误或文件损坏
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
        print_simple_file_list_generic(&list_output.raw_stdout, ui, extractor.name());
    }

    if metrics.total_bytes > 0 {
        ui.info(&format!(
            "共 {} 个文件 ({})",
            metrics.total_files,
            crate::ui::format_bytes(metrics.total_bytes)
        ));
    }

    // 2. 解压
    if metrics.total_bytes > 0 {
        run_extract_with_progress_extractor(
            extractor,
            zip_file,
            temp_folder,
            password,
            metrics,
            start_time,
            ui,
        )
    } else {
        ui.warn("无法获取文件大小，进度不可用");
        extractor.extract(zip_file, temp_folder, password, encoding)?;
        Ok(true)
    }
}

/// 兼容旧接口：直接用路径调用 Bandizip
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
    let extractor = crate::extractor::bandizip::BandizipExtractor::new(seven_zip_path.to_string());
    try_extract_with_extractor(zip_file, temp_folder, password, &extractor, start_time, ui, debug, encoding)
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

fn run_extract_with_progress_extractor(
    extractor: &dyn Extractor,
    zip_file: &str,
    temp_folder: &str,
    password: &str,
    metrics: ArchiveMetrics,
    start_time: Instant,
    ui: &Arc<crate::ui::ConsoleUi>,
) -> anyhow::Result<bool> {
    // 启动解压进程
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

/// 通用文件列表打印（兼容 Bandizip 和 7-Zip 输出）
fn print_simple_file_list_generic(listing: &str, ui: &Arc<crate::ui::ConsoleUi>, extractor_name: &str) {
    // 7-Zip -slt 模式输出需要特殊处理：提取 Path = 值作为文件名
    if extractor_name == "7-Zip" {
        print_7zip_slt_file_list(listing, ui);
        return;
    }

    // Bandizip 等其他引擎：按行过滤输出
    let lines: Vec<&str> = listing
        .lines()
        .map(|l| l.trim())
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("----")
                && !line.starts_with("bz ")
                && !line.contains("Bandizip")
                && !line.contains("7-Zip")
                && !line.contains("Copyright")
                && !line.starts_with("Listing archive:")
                && !line.starts_with("Archive format:")
                && !line.contains("files,")
                && !line.contains("Igor Pavlov")
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

/// 打印 7-Zip -slt 模式的文件列表（提取 Path = 值）
fn print_7zip_slt_file_list(listing: &str, ui: &Arc<crate::ui::ConsoleUi>) {
    let mut entries: Vec<(String, bool)> = Vec::new(); // (path, is_folder)
    let mut current_path: Option<String> = None;
    let mut current_is_folder = false;
    let mut is_archive_header = false;

    for raw_line in listing.lines() {
        let line = raw_line.trim();

        if let Some(path_val) = line.strip_prefix("Path =") {
            let path = path_val.trim();
            // 先保存上一个条目
            if let Some(p) = current_path.take() {
                if !p.is_empty() && !is_archive_header {
                    entries.push((p, current_is_folder));
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

    // 提交最后一个条目
    if let Some(p) = current_path {
        if !p.is_empty() && !is_archive_header {
            entries.push((p, current_is_folder));
        }
    }

    if entries.is_empty() {
        return;
    }

    eprintln!();
    let max_display = 15;
    for (i, (path, is_folder)) in entries.iter().enumerate() {
        if i >= max_display {
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
