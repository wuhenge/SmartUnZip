use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::update;

const DEFAULT_OUTPUT_ENCODING: &str = if cfg!(windows) { "gbk" } else { "utf-8" };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(rename = "OutputEncoding", default = "default_output_encoding")]
    pub output_encoding: String,
    #[serde(rename = "SevenZipPath", default = "default_7zip_path")]
    pub seven_zip_path: String,
    #[serde(rename = "OutputDirectory", default)]
    pub output_directory: String,
    #[serde(rename = "AutoExit", default)]
    pub auto_exit: bool,
    #[serde(rename = "ExtractNestedFolders", default)]
    pub extract_nested_folders: bool,
    #[serde(rename = "DebugMode", default)]
    pub debug_mode: bool,
    #[serde(rename = "DeleteEmptyFolders", default)]
    pub delete_empty_folders: bool,
    #[serde(rename = "FlattenWrapperFolder", default)]
    pub flatten_wrapper_folder: bool,
    #[serde(rename = "DeleteSourceAfterExtract", default)]
    pub delete_source_after_extract: bool,
    #[serde(rename = "OpenFolderAfterExtract", default)]
    pub open_folder_after_extract: bool,
    #[serde(rename = "NestedArchiveDepth", default)]
    pub nested_archive_depth: u32,
    #[serde(rename = "CreateFolderThreshold", default = "default_create_folder_threshold")]
    pub create_folder_threshold: u32,
    #[serde(rename = "Passwords", default)]
    pub passwords: Vec<String>,
    #[serde(rename = "DeleteFiles", default)]
    pub delete_files: Vec<String>,
    #[serde(rename = "DeleteFolders", default)]
    pub delete_folders: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyAppSettings {
    #[serde(rename = "OutputEncoding", default)]
    output_encoding: String,
    #[serde(rename = "SevenZipPath", default)]
    seven_zip_path: String,
    #[serde(rename = "OutputDirectory", default)]
    output_directory: String,
    #[serde(rename = "AutoExit", default)]
    auto_exit: bool,
    #[serde(rename = "ExtractNestedFolders", default)]
    extract_nested_folders: bool,
    #[serde(rename = "DebugMode", default)]
    debug_mode: bool,
    #[serde(rename = "DeleteEmptyFolders", default)]
    delete_empty_folders: bool,
    #[serde(rename = "FlattenWrapperFolder", default)]
    flatten_wrapper_folder: bool,
    #[serde(rename = "DeleteSourceAfterExtract", default)]
    delete_source_after_extract: bool,
    #[serde(rename = "OpenFolderAfterExtract", default)]
    open_folder_after_extract: bool,
    #[serde(rename = "NestedArchiveDepth", default)]
    nested_archive_depth: u32,
    #[serde(rename = "CreateFolderThreshold", default = "default_create_folder_threshold")]
    create_folder_threshold: u32,
    #[serde(rename = "Passwords", default)]
    passwords: Vec<String>,
    #[serde(rename = "DeleteFiles", default)]
    delete_files: Vec<String>,
    #[serde(rename = "DeleteFolders", default)]
    delete_folders: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            output_encoding: DEFAULT_OUTPUT_ENCODING.to_string(),
            seven_zip_path: default_7zip_path(),
            output_directory: String::new(),
            auto_exit: false,
            extract_nested_folders: false,
            debug_mode: false,
            delete_empty_folders: false,
            flatten_wrapper_folder: false,
            delete_source_after_extract: false,
            open_folder_after_extract: false,
            nested_archive_depth: 0,
            create_folder_threshold: 1,
            passwords: vec!["1234".to_string(), "www".to_string(), "1111".to_string()],
            delete_files: vec!["说明.txt".to_string(), "更多资源.url".to_string()],
            delete_folders: vec!["说明".to_string()],
        }
    }
}

impl From<LegacyAppSettings> for AppSettings {
    fn from(legacy: LegacyAppSettings) -> Self {
        Self {
            output_encoding: if legacy.output_encoding.is_empty() {
                DEFAULT_OUTPUT_ENCODING.to_string()
            } else {
                legacy.output_encoding
            },
            seven_zip_path: legacy.seven_zip_path,
            output_directory: legacy.output_directory,
            auto_exit: legacy.auto_exit,
            extract_nested_folders: legacy.extract_nested_folders,
            debug_mode: legacy.debug_mode,
            delete_empty_folders: legacy.delete_empty_folders,
            flatten_wrapper_folder: legacy.flatten_wrapper_folder,
            delete_source_after_extract: legacy.delete_source_after_extract,
            open_folder_after_extract: legacy.open_folder_after_extract,
            nested_archive_depth: legacy.nested_archive_depth,
            create_folder_threshold: legacy.create_folder_threshold,
            passwords: legacy.passwords,
            delete_files: legacy.delete_files,
            delete_folders: legacy.delete_folders,
        }
    }
}

fn default_output_encoding() -> String {
    DEFAULT_OUTPUT_ENCODING.to_string()
}

fn default_create_folder_threshold() -> u32 {
    1
}

#[cfg(windows)]
fn default_7zip_path() -> String {
    r"C:\Program Files\7-Zip\7z.exe".to_string()
}

#[cfg(not(windows))]
fn default_7zip_path() -> String {
    String::new()
}

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    #[serde(rename = "AppSettings")]
    app_settings: AppSettings,
}

#[derive(Deserialize)]
struct LegacyConfigFile {
    #[serde(rename = "AppSettings")]
    app_settings: LegacyAppSettings,
}

fn config_path() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let base_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));

    let direct_path = base_dir.join("appsettings.json");
    if direct_path.exists() {
        return direct_path;
    }

    let mut current = base_dir.to_path_buf();
    while let Some(parent) = current.parent() {
        let candidate = parent.join("appsettings.json");
        if candidate.exists() {
            return candidate;
        }
        current = parent.to_path_buf();
    }

    direct_path
}

#[tauri::command]
pub fn get_config_path() -> String {
    config_path().to_string_lossy().to_string()
}

#[tauri::command]
pub fn load_config() -> Result<AppSettings, String> {
    let config_path = config_path();

    if !config_path.exists() {
        let default_config = ConfigFile {
            app_settings: AppSettings::default(),
        };
        let content = serde_json::to_string_pretty(&default_config)
            .map_err(|e| format!("序列化默认配置失败: {}", e))?;
        std::fs::write(&config_path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;
        return Ok(AppSettings::default());
    }

    let content =
        std::fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    if let Ok(config) = serde_json::from_str::<ConfigFile>(&content) {
        return Ok(config.app_settings);
    }

    let legacy: LegacyConfigFile =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;
    Ok(legacy.app_settings.into())
}

#[tauri::command]
pub fn save_config(settings: AppSettings) -> Result<(), String> {
    let config_path = config_path();
    let config = ConfigFile {
        app_settings: settings,
    };
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, content).map_err(|e| format!("保存配置文件失败: {}", e))?;
    Ok(())
}

#[derive(Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub message: String,
}

#[tauri::command]
pub fn validate_7zip_path(path: String) -> ValidationResult {
    if !std::path::Path::new(&path).exists() {
        return ValidationResult {
            valid: false,
            message: "文件不存在".to_string(),
        };
    }

    let file_name = std::path::Path::new(&path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    if !matches!(file_name.as_str(), "7z.exe" | "7z" | "7zz") {
        return ValidationResult {
            valid: false,
            message: "请选择 7z.exe / 7z / 7zz".to_string(),
        };
    }

    match std::process::Command::new(&path)
        .arg("--help")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            let _ = child.wait();
            ValidationResult {
                valid: true,
                message: "验证成功".to_string(),
            }
        }
        Err(e) => ValidationResult {
            valid: false,
            message: format!("验证失败: {}", e),
        },
    }
}

#[tauri::command]
pub fn check_for_updates() -> update::UpdateInfo {
    update::check_update()
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("无法打开链接: {}", e))
}

#[tauri::command]
pub fn check_context_menu() -> bool {
    crate::registry::is_registered()
}

#[tauri::command]
pub fn add_context_menu() -> Result<(), String> {
    crate::registry::add()
}

#[tauri::command]
pub fn remove_context_menu() -> Result<(), String> {
    crate::registry::remove()
}
