use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::extractor::Extractor;

const MAX_NESTED_DEPTH_LIMIT: u32 = 10;

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
    extractor: &dyn Extractor,
    config: &crate::config::AppSettings,
    ui: &Arc<crate::ui::ConsoleUi>,
) -> Option<String> {
    let max_depth = config.nested_archive_depth.min(MAX_NESTED_DEPTH_LIMIT);
    if current_depth > max_depth + 1 {
        ui.warn(&format!("嵌套深度超限 ({max_depth}层)，停止递归"));
        return None;
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
        return None;
    }

    delete_configured_recursive(temp_folder, config, ui);

    if !Path::new(temp_folder).exists() {
        return None;
    }

    let extracted_path = determine_extracted_path(
        temp_folder,
        output_folder,
        zip_file,
        current_depth,
        passwords,
        extractor,
        config,
        ui,
    );

    if let Some(ref path) = extracted_path {
        if config.flatten_wrapper_folder {
            if let Some(new_path) = flatten_wrapper_folder(path, ui) {
                let _ = std::fs::remove_dir(path);
                if config.delete_empty_folders {
                    delete_empty_folders(&new_path, ui);
                }
                return Some(new_path);
            }
        }
        if config.delete_empty_folders {
            delete_empty_folders(path, ui);
        }
    }

    extracted_path
}

fn determine_extracted_path(
    temp_folder: &str,
    output_folder: &str,
    zip_file: &str,
    current_depth: u32,
    passwords: &[String],
    extractor: &dyn Extractor,
    config: &crate::config::AppSettings,
    ui: &Arc<crate::ui::ConsoleUi>,
) -> Option<String> {
    let top = list_top_entries(temp_folder);

    if config.nested_archive_depth > 0 {
        let archive_to_check: Option<String> = if top.files.len() == 1 && top.dirs.is_empty() {
            let ext = Path::new(&top.files[0])
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase());
            if ext
                .as_deref()
                .map_or(false, |e| matches!(e, "zip" | "rar" | "tar" | "7z" | "7z1"))
            {
                Some(top.files[0].clone())
            } else {
                None
            }
        } else if config.extract_nested_folders && !top.dirs.is_empty() {
            let all_files = match walk_all_files(temp_folder) {
                Some(f) if f.len() == 1 => f,
                _ => {
                    return Some(move_as_extracted_folder(temp_folder, output_folder, zip_file));
                }
            };
            let ext = Path::new(&all_files[0])
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase());
            if ext
                .as_deref()
                .map_or(false, |e| matches!(e, "zip" | "rar" | "tar" | "7z" | "7z1"))
            {
                Some(all_files[0].clone())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(archive) = archive_to_check {
            let archive_name = Path::new(&archive)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            return handle_nested_archive(
                &archive,
                &archive_name,
                temp_folder,
                output_folder,
                zip_file,
                current_depth,
                passwords,
                extractor,
                config,
                ui,
            );
        }
    }

    let top = list_top_entries(temp_folder);
    
    let extracted_path = if top.dirs.is_empty() {
        let file_count = top.files.len();
        let should_create_folder = config.create_folder_threshold > 0 
            && file_count > config.create_folder_threshold as usize;
        
        if should_create_folder {
            move_as_extracted_folder(temp_folder, output_folder, zip_file)
        } else if file_count == 1 {
            let file_path = &top.files[0];
            let file_name = Path::new(file_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            move_single_file(file_path, temp_folder, output_folder, &file_name);
            output_folder.to_string()
        } else {
            move_files_to_output(temp_folder, output_folder);
            output_folder.to_string()
        }
    } else if top.dirs.len() == 1 {
        if config.extract_nested_folders {
            let dest = move_folder_contents_with_return(&top.dirs[0], temp_folder, output_folder);
            dest
        } else {
            move_single_directory(&top.dirs[0], temp_folder, output_folder)
        }
    } else {
        move_as_extracted_folder(temp_folder, output_folder, zip_file)
    };

    Some(extracted_path)
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
    extractor: &dyn Extractor,
    config: &crate::config::AppSettings,
    ui: &Arc<crate::ui::ConsoleUi>,
) -> Option<String> {
    let max_depth = config.nested_archive_depth.min(MAX_NESTED_DEPTH_LIMIT);
    if current_depth > max_depth + 1 {
        ui.warn(&format!("嵌套深度超限 ({max_depth}层): {archive_name}"));
        return None;
    }

    ui.info(&format!(
        "发现嵌套包 [{current_depth}/{max_depth}] {archive_name}"
    ));

    let nested_temp_folder = format!("{temp_folder}_nested");
    let _ = std::fs::create_dir_all(&nested_temp_folder);

    for (idx, pwd) in passwords.iter().enumerate() {
        let debug_cmd = if config.debug_mode {
            Some(format!("{} list -y {}", extractor.exe_path(), archive_path))
        } else {
            None
        };
        ui.attempt_password(idx + 1, passwords.len(), pwd, debug_cmd.as_deref());
        let start = std::time::Instant::now();

        match crate::archive::try_extract_with_extractor(
            archive_path,
            &nested_temp_folder,
            pwd,
            extractor,
            start,
            ui,
            config.debug_mode,
            &config.output_encoding,
        ) {
            Ok(true) => {
                let _ = std::fs::remove_file(archive_path);
                
                move_nested_contents(&nested_temp_folder, temp_folder);
                try_delete_directory(&nested_temp_folder);
                
                return process_temp_folder(
                    temp_folder,
                    output_folder,
                    zip_file,
                    current_depth + 1,
                    passwords,
                    extractor,
                    config,
                    ui,
                );
            }
            _ => continue,
        }
    }
    
    try_delete_directory(&nested_temp_folder);
    ui.warn(&format!("嵌套包解压失败: {archive_name}"));
    None
}

fn move_nested_contents(src: &str, dst: &str) {
    if !Path::new(src).exists() {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(src) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let name = src_path.file_name().unwrap_or_default();
            let dst_path = Path::new(dst).join(name);
            
            if src_path.is_dir() {
                let _ = std::fs::rename(&src_path, &dst_path);
            } else {
                let _ = std::fs::rename(&src_path, &dst_path);
            }
        }
    }
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

fn move_single_directory(dir: &str, temp_folder: &str, output_folder: &str) -> String {
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
    dest.to_string_lossy().to_string()
}

fn move_files_to_output(temp_folder: &str, output_folder: &str) {
    if let Ok(entries) = std::fs::read_dir(temp_folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let dest = get_unique_path(output_folder, &name);
                let _ = std::fs::rename(&path, &dest);
            }
        }
    }
    try_delete_directory(temp_folder);
}

fn move_folder_contents_with_return(
    dir: &str,
    temp_folder: &str,
    output_folder: &str,
) -> String {
    let mut moved_paths = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let dest = get_unique_path(output_folder, &name);
            let _ = std::fs::rename(&path, &dest);
            moved_paths.push(dest.to_string_lossy().to_string());
        }
    }
    try_delete_directory(temp_folder);
    
    if moved_paths.len() == 1 {
        let first = &moved_paths[0];
        if Path::new(first).is_dir() {
            return first.clone();
        }
    }
    output_folder.to_string()
}

fn move_as_extracted_folder(temp_folder: &str, output_folder: &str, zip_file: &str) -> String {
    let stem = Path::new(zip_file)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let dest = get_unique_path(output_folder, &stem);
    let _ = std::fs::rename(temp_folder, &dest);
    dest.to_string_lossy().to_string()
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

fn flatten_wrapper_folder(dir: &str, ui: &Arc<crate::ui::ConsoleUi>) -> Option<String> {
    let path = Path::new(dir);
    if !path.exists() || !path.is_dir() {
        return None;
    }

    let parent = path.parent()?;
    let entries: Vec<_> = std::fs::read_dir(path).ok()?.flatten().collect();
    
    if entries.len() != 1 {
        return None;
    }
    
    let entry = &entries[0];
    let entry_path = entry.path();
    
    if !entry_path.is_dir() {
        return None;
    }
    
    let folder_name = entry_path.file_name()?.to_string_lossy().to_string();
    let dest = parent.join(&folder_name);
    let temp_dest = parent.join(format!("_{folder_name}_tmp"));
    
    if dest.exists() {
        let _ = std::fs::rename(&entry_path, &temp_dest);
        let _ = std::fs::remove_dir_all(&dest);
        let _ = std::fs::rename(&temp_dest, &dest);
    } else if let Err(e) = std::fs::rename(&entry_path, &dest) {
        ui.warn(&format!("提升文件夹失败: {}", e));
        return None;
    }
    
    ui.info(&format!("提升文件夹: {}", folder_name));
    Some(dest.to_string_lossy().to_string())
}

fn delete_empty_folders(dir: &str, ui: &Arc<crate::ui::ConsoleUi>) {
    let path = Path::new(dir);
    if !path.exists() || !path.is_dir() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                delete_empty_folders(&entry_path.to_string_lossy(), ui);
                if is_dir_empty(&entry_path) {
                    let name = entry_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    ui.info(&format!("删除空文件夹: {}", name));
                    let _ = std::fs::remove_dir(&entry_path);
                }
            }
        }
    }
}

fn is_dir_empty(path: &Path) -> bool {
    match std::fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    }
}

pub fn print_directory_tree(dir: &str, ui: &Arc<crate::ui::ConsoleUi>) {
    let path = Path::new(dir);
    if !path.exists() {
        return;
    }

    ui.debug_section("解压结果");
    let mut state = TreeState::default();
    print_tree(path, "", true, ui, &mut state);
}

#[derive(Default)]
struct TreeState {
    count: usize,
    max_reached: bool,
}

impl TreeState {
    const MAX_ITEMS: usize = 20;

    fn can_show(&mut self) -> bool {
        if self.count >= Self::MAX_ITEMS {
            self.max_reached = true;
            return false;
        }
        self.count += 1;
        true
    }
}

fn print_tree(
    path: &Path,
    prefix: &str,
    is_last: bool,
    ui: &Arc<crate::ui::ConsoleUi>,
    state: &mut TreeState,
) {
    use colored::Colorize;

    if !state.can_show() {
        return;
    }

    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let connector = if is_last { "└── " } else { "├── " };
    let display_name = if path.is_dir() {
        format!("{}{}", "📁 ".cyan(), name.cyan().bold())
    } else {
        format!("{}{}", "📄 ".dimmed(), name.white())
    };
    eprintln!("{}{}{}", prefix.dimmed(), connector.dimmed(), display_name);

    if path.is_dir() {
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        if let Ok(entries) = std::fs::read_dir(path) {
            let mut entries: Vec<_> = entries.flatten().collect();
            entries.sort_by(|a, b| {
                let a_is_dir = a.path().is_dir();
                let b_is_dir = b.path().is_dir();
                b_is_dir
                    .cmp(&a_is_dir)
                    .then_with(|| a.file_name().cmp(&b.file_name()))
            });

            let len = entries.len();
            for (i, entry) in entries.iter().enumerate() {
                if state.max_reached {
                    break;
                }
                print_tree(&entry.path(), &new_prefix, i == len - 1 && !state.max_reached, ui, state);
            }

            if state.max_reached && state.count == TreeState::MAX_ITEMS {
                let omitted_connector = if is_last { "    " } else { "│   " };
                eprintln!(
                    "{}{}{}",
                    prefix.dimmed(),
                    omitted_connector.dimmed(),
                    "... 省略更多内容".dimmed().italic()
                );
                state.count += 1;
            }
        }
    }
}
