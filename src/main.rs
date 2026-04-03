mod archive;
mod config;
mod files;
mod registry;
mod ui;

use std::io::Write;
use std::sync::Arc;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_RELEASES_URL: &str = "https://github.com/wuhenge/SmartUnZip/releases";
const GITHUB_API_URL: &str = "https://api.github.com/repos/wuhenge/SmartUnZip/releases/latest";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ui = Arc::new(ui::ConsoleUi::new("SmartUnZip"));

    // 启动时加载配置（首次运行自动生成配置文件）
    let settings = match config::load() {
        Ok(s) => s,
        Err(e) => {
            ui.error(&format!("{e}"));
            return;
        }
    };

    if args.is_empty() {
        ui.header("设置");

        let registered = registry::is_registered();
        let status = if registered { "已安装" } else { "未安装" };
        let toggle_label = if registered {
            "移除右键菜单"
        } else {
            "添加右键菜单"
        };
        ui.info(&format!("右键菜单: {status}"));
        ui.info(&format!("Bandizip: {}", settings.seven_zip_path));
        eprintln!();
        eprintln!("  1. {toggle_label}");
        eprintln!("  2. 验证 Bandizip");
        eprintln!("  3. 检查更新");
        eprintln!("  0. 退出");
        eprintln!();
        eprint!("  请选择: ");
        std::io::stderr().flush().ok();

        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
        let choice = input.trim();

        match choice {
            "1" => {
                if registered {
                    registry::remove(&ui);
                } else {
                    registry::add(&ui);
                }
            }
            "2" => verify_bandizip(&settings.seven_zip_path, &ui),
            "3" => check_for_updates(&ui),
            _ => {}
        }

        eprintln!();
        wait_key();
        return;
    }

    if !std::path::Path::new(&settings.seven_zip_path).exists() {
        ui.error(&format!("未找到 Bandizip: {}", settings.seven_zip_path));
        return;
    }

    if settings.debug_mode {
        let width = 79;
        let sep = "=".repeat(width);
        eprintln!("{}", sep);
        eprintln!("  SmartUnZip  调试信息");
        eprintln!("{}", sep);
        eprintln!("  [*] Bandizip路径: {}", settings.seven_zip_path);
        eprintln!("  [*] 解压嵌套压缩包: {}", settings.extract_nested_archives);
        eprintln!("  [*] 嵌套压缩包深度: {}", settings.nested_archive_depth);
        eprintln!("  [*] 展平嵌套文件夹: {}", settings.extract_nested_folders);
        eprintln!("  [*] 自动退出: {}", settings.auto_exit);
        eprintln!("  [*] 删除空文件夹: {}", settings.delete_empty_folders);
        eprintln!("  [*] 密码数量: {}", settings.passwords.len());
        eprintln!();
    }

    for zip_file in &args {
        let output_folder = std::path::Path::new(zip_file)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let file_name = std::path::Path::new(zip_file)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        ui.header(&format!("解压 {file_name}"));

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
                let pwd_flag = format!("-p:{pwd}");
                Some(format!("l -list:v -y {pwd_flag} {zip_file}"))
            } else {
                None
            };
            ui.attempt_password(
                pwd_idx + 1,
                settings.passwords.len(),
                pwd,
                debug_cmd.as_deref(),
            );

            match archive::try_extract(
                zip_file,
                &temp_folder,
                pwd,
                &settings.seven_zip_path,
                start_time,
                &ui,
                settings.debug_mode,
            ) {
                Ok(true) => {
                    let extracted_path = files::process_temp_folder(
                        &temp_folder,
                        &output_folder,
                        zip_file,
                        1,
                        &settings.passwords,
                        &settings.seven_zip_path,
                        &settings,
                        &ui,
                    );
                    extracted = true;
                    ui.success("解压完成");
                    if settings.debug_mode {
                        if let Some(path) = extracted_path {
                            files::print_directory_tree(&path, &ui);
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    break; // 修复原 C# 中的 bug：原为 return，跳过后续文件
                }
                _ => {
                    files::try_delete_directory(&temp_folder);
                }
            }
        }

        if !extracted {
            ui.error("所有密码均失败，解压终止");
            wait_key();
        }
    }

    if !settings.auto_exit {
        wait_key();
    }
}

fn wait_key() {
    eprint!("  按回车键退出...");
    std::io::stderr().flush().ok();
    let _ = std::io::stdin().read_line(&mut String::new());
}

fn verify_bandizip(path: &str, ui: &Arc<ui::ConsoleUi>) {
    eprintln!();

    let file_name = std::path::Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    if file_name != "bz.exe" {
        ui.error(&format!(
            "应使用 bz.exe 而非 {}",
            std::path::Path::new(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
        return;
    }

    if !std::path::Path::new(path).exists() {
        ui.error("文件不存在");
        return;
    }

    match std::process::Command::new(path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            let _ = child.wait();
            ui.success("验证成功");
        }
        Err(e) => {
            ui.error(&format!("验证失败: {e}"));
        }
    }
}

fn check_for_updates(ui: &Arc<ui::ConsoleUi>) {
    eprintln!();
    ui.info(&format!("当前版本: v{VERSION}"));
    ui.info("正在检查更新...");

    match fetch_latest_version() {
        Ok(latest_version) => {
            eprintln!();
            if compare_versions(VERSION, &latest_version) {
                ui.success(&format!("发现新版本: v{latest_version}"));
                eprintln!();
                ui.info("请访问以下链接下载最新版本：");
                eprintln!("  {GITHUB_RELEASES_URL}");
            } else {
                ui.success("当前版本已是最新！");
                eprintln!();
                ui.info("项目地址：");
                eprintln!("  {GITHUB_RELEASES_URL}");
            }
        }
        Err(e) => {
            eprintln!();
            ui.error(&format!("检查更新失败: {e}"));
            eprintln!();
            ui.info("请手动访问以下链接检查更新：");
            eprintln!("  {GITHUB_RELEASES_URL}");
        }
    }
}

fn fetch_latest_version() -> Result<String, String> {
    let response = ureq::get(GITHUB_API_URL)
        .set("User-Agent", &format!("SmartUnZip/{VERSION}"))
        .set("Accept", "application/vnd.github.v3+json")
        .call()
        .map_err(|e| format!("网络请求失败: {e}"))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("解析响应失败: {e}"))?;

    let tag_name = json["tag_name"]
        .as_str()
        .ok_or("无法获取版本信息")?;

    let version = tag_name.trim_start_matches('v').to_string();
    Ok(version)
}

fn compare_versions(current: &str, latest: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let current_parts = parse_version(current);
    let latest_parts = parse_version(latest);

    for i in 0..std::cmp::max(current_parts.len(), latest_parts.len()) {
        let current_val = current_parts.get(i).unwrap_or(&0);
        let latest_val = latest_parts.get(i).unwrap_or(&0);

        if latest_val > current_val {
            return true;
        } else if latest_val < current_val {
            return false;
        }
    }

    false
}
