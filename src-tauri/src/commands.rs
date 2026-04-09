use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::update;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(rename = "SevenZipPath")]
    pub seven_zip_path: String,
    #[serde(rename = "AutoExit")]
    pub auto_exit: bool,
    #[serde(rename = "ExtractNestedFolders")]
    pub extract_nested_folders: bool,
    #[serde(rename = "DebugMode")]
    pub debug_mode: bool,
    #[serde(rename = "DeleteEmptyFolders")]
    pub delete_empty_folders: bool,
    #[serde(rename = "FlattenWrapperFolder")]
    pub flatten_wrapper_folder: bool,
    #[serde(rename = "DeleteSourceAfterExtract")]
    pub delete_source_after_extract: bool,
    #[serde(rename = "OpenFolderAfterExtract")]
    pub open_folder_after_extract: bool,
    #[serde(rename = "NestedArchiveDepth")]
    pub nested_archive_depth: u32,
    #[serde(rename = "CreateFolderThreshold")]
    pub create_folder_threshold: u32,
    #[serde(rename = "Passwords")]
    pub passwords: Vec<String>,
    #[serde(rename = "DeleteFiles")]
    pub delete_files: Vec<String>,
    #[serde(rename = "DeleteFolders")]
    pub delete_folders: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            seven_zip_path: r"C:\Program Files\Bandizip\bz.exe".to_string(),
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

#[derive(Serialize, Deserialize)]
struct ConfigFile {
    #[serde(rename = "AppSettings")]
    app_settings: AppSettings,
}

fn config_path() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let base_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    
    let config_path = base_dir.join("appsettings.json");
    if config_path.exists() {
        return config_path;
    }
    
    let mut current = base_dir.to_path_buf();
    while let Some(parent) = current.parent() {
        let candidate = parent.join("appsettings.json");
        if candidate.exists() {
            return candidate;
        }
        current = parent.to_path_buf();
    }
    
    config_path
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

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    let config: ConfigFile = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;
    
    Ok(config.app_settings)
}

#[tauri::command]
pub fn save_config(settings: AppSettings) -> Result<(), String> {
    let config_path = config_path();
    let config = ConfigFile {
        app_settings: settings,
    };
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, content)
        .map_err(|e| format!("保存配置文件失败: {}", e))?;
    Ok(())
}

#[derive(Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub message: String,
}

#[tauri::command]
pub fn validate_bandizip_path(path: String) -> ValidationResult {
    let file_name = std::path::Path::new(&path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    if file_name != "bz.exe" {
        return ValidationResult {
            valid: false,
            message: format!("应选择 bz.exe 而非 {}", 
                std::path::Path::new(&path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            ),
        };
    }

    if !std::path::Path::new(&path).exists() {
        return ValidationResult {
            valid: false,
            message: "文件不存在".to_string(),
        };
    }

    match std::process::Command::new(&path)
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
