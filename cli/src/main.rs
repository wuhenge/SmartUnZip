mod archive;
mod config;
mod extractor;
mod files;
mod ui;

use std::io::Write;
use std::sync::Arc;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ui = Arc::new(ui::ConsoleUi::new("SmartUnZip"));

    // 启动时加载配置
    let settings = match config::load() {
        Ok(s) => s,
        Err(e) => {
            ui.error(&format!("{e}"));
            wait_key();
            return;
        }
    };

    // 创建解压引擎
    let extractor = match config::create_extractor_from_config(&settings) {
        Some(e) => e,
        None => {
            ui.error("未找到可用的解压工具，请在配置文件中指定引擎类型和路径");
            wait_key();
            return;
        }
    };

    if args.is_empty() {
        // 验证配置是否有效
        let exe_path = extractor.exe_path();
        if !exe_path.is_empty() && std::path::Path::new(exe_path).exists() && extractor.validate(exe_path) {
            ui.success(&format!("配置有效: {} ({})", extractor.name(), exe_path));
        } else if exe_path.is_empty() {
            ui.error("配置无效: 未配置解压工具路径");
        } else if !std::path::Path::new(exe_path).exists() {
            ui.error(&format!("配置无效: 文件不存在 {}", exe_path));
        } else {
            ui.error(&format!("配置无效: 文件名不匹配 {} 的可执行文件", extractor.name()));
        }

        wait_key();
        return;
    }

    if !std::path::Path::new(extractor.exe_path()).exists() {
        ui.error(&format!(
            "未找到 {}: {}",
            extractor.name(),
            extractor.exe_path()
        ));
        if !settings.auto_exit {
            wait_key();
        }
        return;
    }

    if settings.debug_mode {
        ui.print_config(&settings, extractor.as_ref());
    }

    for zip_file in &args {
        let output_folder = if settings.output_directory.is_empty() {
            std::path::Path::new(zip_file)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        } else {
            let dir = &settings.output_directory;
            if !std::path::Path::new(dir).exists() {
                if let Err(e) = std::fs::create_dir_all(dir) {
                    ui.warn(&format!("创建输出目录失败: {}", e));
                }
            }
            dir.clone()
        };
        let file_name = std::path::Path::new(zip_file)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        ui.header(&format!("解压 {file_name} [{}]", extractor.name()));

        let mut extracted = false;
        let start_time = std::time::Instant::now();

        for (pwd_idx, pwd) in settings.passwords.iter().enumerate() {
            let stem = std::path::Path::new(&file_name)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            let temp_folder = std::path::Path::new(&output_folder)
                .join(format!("BzTemp_{stem}"))
                .to_string_lossy()
                .to_string();
            files::try_delete_directory(&temp_folder);
            let _ = std::fs::create_dir_all(&temp_folder);

            let debug_cmd = if settings.debug_mode {
                Some(format!("{} list -y {}", extractor.exe_path(), zip_file))
            } else {
                None
            };
            ui.attempt_password(
                pwd_idx + 1,
                settings.passwords.len(),
                pwd,
                debug_cmd.as_deref(),
            );

            match archive::try_extract_with_extractor(
                zip_file,
                &temp_folder,
                pwd,
                extractor.as_ref(),
                start_time,
                &ui,
                settings.debug_mode,
                &settings.output_encoding,
            ) {
                Ok(true) => {
                    let extracted_path = files::process_temp_folder(
                        &temp_folder,
                        &output_folder,
                        zip_file,
                        1,
                        &settings.passwords,
                        extractor.as_ref(),
                        &settings,
                        &ui,
                    );
                    extracted = true;
                    ui.success("解压完成");
                    
                    if settings.delete_source_after_extract {
                        if let Err(e) = std::fs::remove_file(zip_file) {
                            ui.warn(&format!("删除源文件失败: {}", e));
                        } else {
                            ui.info(&format!("已删除源文件: {}", file_name));
                        }
                    }
                    
                    if settings.open_folder_after_extract {
                        if let Some(ref path) = extracted_path {
                            if let Err(e) = open_folder(path) {
                                ui.warn(&format!("打开文件夹失败: {}", e));
                            }
                        }
                    }
                    
                    if settings.debug_mode {
                        if let Some(path) = extracted_path {
                            files::print_directory_tree(&path, &ui);
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    break;
                }
                _ => {
                    files::try_delete_directory(&temp_folder);
                }
            }
        }

        if !extracted {
            ui.error("所有密码均失败，解压终止");
        }
    }

    if !settings.auto_exit {
        wait_key();
    }
}

fn wait_key() {
    eprintln!();
    eprint!("  按回车键退出...");
    std::io::stderr().flush().ok();
    let _ = std::io::stdin().read_line(&mut String::new());
}

fn open_folder(path: &str) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("无法打开文件夹: {}", e))
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("无法打开文件夹: {}", e))
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("无法打开文件夹: {}", e))
    }

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        Err(anyhow::anyhow!("此平台不支持自动打开文件夹"))
    }
}
