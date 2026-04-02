use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    #[serde(rename = "SevenZipPath", default)]
    pub seven_zip_path: String,
    #[serde(rename = "Passwords", default)]
    pub passwords: Vec<String>,
    #[serde(rename = "DeleteFiles", default)]
    pub delete_files: Vec<String>,
    #[serde(rename = "DeleteFolders", default)]
    pub delete_folders: Vec<String>,
    #[serde(rename = "ExtractNestedArchives", default = "default_extract_nested")]
    pub extract_nested_archives: bool,
    #[serde(rename = "NestedArchiveDepth", default = "default_nested_depth")]
    pub nested_archive_depth: u32,
    #[serde(rename = "AutoExit", default = "default_auto_exit")]
    pub auto_exit: bool,
    #[serde(
        rename = "ExtractNestedFolders",
        default = "default_extract_nested_folders"
    )]
    pub extract_nested_folders: bool,
    #[serde(rename = "DebugMode", default = "default_debug_mode")]
    pub debug_mode: bool,
    #[serde(
        rename = "DeleteEmptyFolders",
        default = "default_delete_empty_folders"
    )]
    pub delete_empty_folders: bool,
    #[serde(rename = "CreateFolderThreshold", default = "default_create_folder_threshold")]
    pub create_folder_threshold: u32,
    #[serde(rename = "FlattenWrapperFolder", default = "default_flatten_wrapper_folder")]
    pub flatten_wrapper_folder: bool,
}

fn default_extract_nested() -> bool {
    false
}

fn default_nested_depth() -> u32 {
    1
}

fn default_auto_exit() -> bool {
    false
}

fn default_extract_nested_folders() -> bool {
    false
}

fn default_debug_mode() -> bool {
    false
}

fn default_delete_empty_folders() -> bool {
    false
}

fn default_create_folder_threshold() -> u32 {
    1
}

fn default_flatten_wrapper_folder() -> bool {
    false
}

#[derive(Deserialize)]
struct ConfigFile {
    #[serde(rename = "AppSettings")]
    app_settings: AppSettings,
}

#[cfg(windows)]
const DEFAULT_CONFIG: &str = r#"{
  "AppSettings": {
    "SevenZipPath": "C:\\Program Files\\Bandizip\\bz.exe",
    "Passwords": [
      "1234",
      "www",
      "1111"
    ],
    "DeleteFiles": [
      "说明.txt",
      "更多资源.url"
    ],
    "DeleteFolders": [
      "说明"
    ],
    "ExtractNestedArchives": false,
    "NestedArchiveDepth": 1,
    "AutoExit": false,
    "ExtractNestedFolders": false,
    "DebugMode": false,
    "DeleteEmptyFolders": false,
    "CreateFolderThreshold": 1,
    "FlattenWrapperFolder": false
  }
}
"#;

#[cfg(not(windows))]
const DEFAULT_CONFIG: &str = r#"{
  "AppSettings": {
    "SevenZipPath": "",
    "Passwords": [
      "1234",
      "www",
      "1111"
    ],
    "DeleteFiles": [
      "说明.txt",
      "更多资源.url"
    ],
    "DeleteFolders": [
      "说明"
    ],
    "ExtractNestedArchives": false,
    "NestedArchiveDepth": 1,
    "AutoExit": false,
    "ExtractNestedFolders": false,
    "DebugMode": false,
    "DeleteEmptyFolders": false,
    "CreateFolderThreshold": 1,
    "FlattenWrapperFolder": false
  }
}
"#;

pub fn load() -> anyhow::Result<AppSettings> {
    let exe_path = std::env::current_exe()?;
    let default_dir = PathBuf::from(".");
    let base_dir = exe_path.parent().unwrap_or(&default_dir);
    let config_path = base_dir.join("appsettings.json");

    if !config_path.exists() {
        std::fs::write(&config_path, DEFAULT_CONFIG)
            .map_err(|e| anyhow::anyhow!("无法创建配置文件 {}: {}", config_path.display(), e))?;
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("无法读取配置文件 {}: {}", config_path.display(), e))?;

    let config: ConfigFile =
        serde_json::from_str(&content).map_err(|e| anyhow::anyhow!("配置文件解析失败: {}", e))?;

    Ok(config.app_settings)
}
