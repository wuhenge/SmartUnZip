use std::path::{Path, PathBuf};
use std::sync::Arc;

const MAX_NESTED_DEPTH: u32 = 5;

pub fn is_archive_file(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext = ext.to_ascii_lowercase();
            matches!(ext.as_str(), "zip" | "rar" | "tar" | "7z" | "7z1" | "txt")
        })
        .unwrap_or(false)
}

pub fn delete_file(
    path: &str,
    file_name: &str,
    keywords: &[String],
    ui: &Arc<crate::ui::ConsoleUi>,
) {
    if !Path::new(path).exists() {
        return;
    }
    for keyword in keywords {
        if file_name.contains(keyword.as_str()) {
            if let Err(e) = std::fs::remove_file(path) {
                ui.warn(&format!("删除文件失败 {file_name}: {e}"));
            } else {
                ui.info(&format!("删除文件: {file_name}"));
            }
            return;
        }
    }
}

pub fn delete_folder(
    path: &str,
    folder_name: &str,
    keywords: &[String],
    ui: &Arc<crate::ui::ConsoleUi>,
) {
    if !Path::new(path).exists() {
        return;
    }
    for keyword in keywords {
        if folder_name.contains(keyword.as_str()) {
            if let Err(e) = std::fs::remove_dir_all(path) {
                ui.warn(&format!("删除文件夹失败 {folder_name}: {e}"));
            } else {
                ui.info(&format!("删除文件夹: {folder_name}"));
            }
            return;
        }
    }
}

pub fn try_delete_directory(path: &str) {
    let _ = std::fs::remove_dir_all(path);
}

pub fn process_temp_folder(
    temp_folder: &str,
    output_folder: &str,
    zip_file: &str,
    current_depth: u32,
    passwords: &[String],
    seven_zip_path: &str,
    config: &crate::config::AppSettings,
    ui: &Arc<crate::ui::ConsoleUi>,
) {
    if current_depth > MAX_NESTED_DEPTH {
        ui.warn(&format!("嵌套深度超限 ({MAX_NESTED_DEPTH}层)，停止递归"));
        return;
    }

    if let Ok(entries) = std::fs::read_dir(temp_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                delete_file(&path.to_string_lossy(), &name, &config.delete_files, ui);
            } else if path.is_dir() {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                delete_folder(&path.to_string_lossy(), &name, &config.delete_folders, ui);
            }
        }
    }

    if !Path::new(temp_folder).exists() {
        return;
    }

    delete_configured_recursive(temp_folder, config, ui);

    if !Path::new(temp_folder).exists() {
        return;
    }

    let all_files = match walk_all_files(temp_folder) {
        Some(f) if !f.is_empty() => f,
        _ => return,
    };

    if all_files.len() == 1 && is_archive_file(&all_files[0]) {
        let archive = &all_files[0];
        let archive_name = Path::new(archive)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        handle_nested_archive(
            archive,
            &archive_name,
            temp_folder,
            output_folder,
            zip_file,
            current_depth,
            passwords,
            seven_zip_path,
            config,
            ui,
        );
        return;
    }

    let top = list_top_entries(temp_folder);
    if top.files.len() == 1 && top.dirs.is_empty() {
        let file_path = &top.files[0];
        let file_name = Path::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        move_single_file(file_path, temp_folder, output_folder, &file_name);
    } else if top.files.is_empty() && top.dirs.len() == 1 {
        move_single_directory(&top.dirs[0], temp_folder, output_folder);
    } else {
        move_as_extracted_folder(temp_folder, output_folder, zip_file);
    }
}

fn delete_configured_recursive(
    dir: &str,
    config: &crate::config::AppSettings,
    ui: &Arc<crate::ui::ConsoleUi>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            delete_file(&path.to_string_lossy(), &name, &config.delete_files, ui);
        } else if path.is_dir() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            delete_folder(&path.to_string_lossy(), &name, &config.delete_folders, ui);
            if path.exists() {
                delete_configured_recursive(&path.to_string_lossy(), config, ui);
            }
        }
    }
}

fn handle_nested_archive(
    archive_path: &str,
    archive_name: &str,
    temp_folder: &str,
    output_folder: &str,
    zip_file: &str,
    current_depth: u32,
    passwords: &[String],
    seven_zip_path: &str,
    config: &crate::config::AppSettings,
    ui: &Arc<crate::ui::ConsoleUi>,
) {
    if current_depth > MAX_NESTED_DEPTH {
        ui.warn(&format!(
            "嵌套深度超限 ({MAX_NESTED_DEPTH}层): {archive_name}"
        ));
        return;
    }

    ui.info(&format!(
        "发现嵌套包 [{current_depth}/{MAX_NESTED_DEPTH}] {archive_name}"
    ));

    for (idx, pwd) in passwords.iter().enumerate() {
        ui.attempt_password(idx + 1, passwords.len(), pwd);
        let start = std::time::Instant::now();

        match crate::archive::try_extract(archive_path, temp_folder, pwd, seven_zip_path, start, ui)
        {
            Ok(true) => {
                let _ = std::fs::remove_file(archive_path);
                process_temp_folder(
                    temp_folder,
                    output_folder,
                    zip_file,
                    current_depth + 1,
                    passwords,
                    seven_zip_path,
                    config,
                    ui,
                );
                return;
            }
            _ => continue,
        }
    }

    ui.warn(&format!("嵌套包解压失败: {archive_name}"));
}

struct TopEntries {
    files: Vec<String>,
    dirs: Vec<String>,
}

fn list_top_entries(dir: &str) -> TopEntries {
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            if path.is_file() {
                files.push(path_str);
            } else if path.is_dir() {
                dirs.push(path_str);
            }
        }
    }

    TopEntries { files, dirs }
}

fn walk_all_files(dir: &str) -> Option<Vec<String>> {
    let mut result = Vec::new();
    fn walk(path: &Path, out: &mut Vec<String>) {
        let entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.push(p.to_string_lossy().to_string());
            }
        }
    }
    walk(Path::new(dir), &mut result);
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn move_single_file(file_path: &str, temp_folder: &str, output_folder: &str, file_name: &str) {
    if !Path::new(file_path).exists() {
        return;
    }
    let dest = Path::new(output_folder).join(file_name);
    let _ = std::fs::rename(file_path, &dest);
    try_delete_directory(temp_folder);
}

fn move_single_directory(dir: &str, temp_folder: &str, output_folder: &str) {
    let mut current = dir.to_string();

    loop {
        let entries = list_top_entries(&current);
        if entries.dirs.len() == 1 && entries.files.is_empty() {
            current = entries.dirs[0].clone();
        } else {
            break;
        }
    }

    let dir_name = Path::new(&current)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let dir_name = if dir_name.contains('[') && dir_name.contains(']') && !dir_name.contains("] ") {
        dir_name.replace("]", "] ").trim().to_string()
    } else {
        dir_name
    };

    let dest = get_unique_path(output_folder, &dir_name);
    let _ = std::fs::rename(&current, &dest);
    try_delete_directory(temp_folder);
}

fn move_as_extracted_folder(temp_folder: &str, output_folder: &str, zip_file: &str) {
    let stem = Path::new(zip_file)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let dest = get_unique_path(output_folder, &stem);
    let _ = std::fs::rename(temp_folder, &dest);
}

fn get_unique_path(output_folder: &str, name: &str) -> PathBuf {
    let mut path = PathBuf::from(output_folder).join(name);
    if !path.exists() {
        return path;
    }

    let mut suffix = 1u32;
    loop {
        path = PathBuf::from(output_folder).join(format!("{name}_{suffix}"));
        if !path.exists() {
            return path;
        }
        suffix += 1;
    }
}
