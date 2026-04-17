use serde::Deserialize;
use std::path::PathBuf;

/// 默认解压引擎类型，修改此值即可切换默认引擎
const DEFAULT_EXTRACTOR_TYPE: &str = "7zip";
/// 默认输出编码
const DEFAULT_OUTPUT_ENCODING: &str = "gbk";

#[derive(Debug, Deserialize, Clone)]
pub struct AppSettings {
    #[serde(rename = "ExtractorType", default = "default_extractor_type")]
    pub extractor_type: String,
    #[serde(rename = "OutputEncoding", default = "default_output_encoding")]
    pub output_encoding: String,
    #[serde(rename = "SevenZipPath", default)]
    pub seven_zip_path: String,
    #[serde(rename = "SevenZipPath7z", default)]
    pub seven_zip_path_7z: String,
    #[serde(rename = "OutputDirectory", default)]
    pub output_directory: String,
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
    #[serde(rename = "FlattenWrapperFolder", default = "default_flatten_wrapper_folder")]
    pub flatten_wrapper_folder: bool,
    #[serde(rename = "DeleteSourceAfterExtract", default = "default_delete_source_after_extract")]
    pub delete_source_after_extract: bool,
    #[serde(rename = "OpenFolderAfterExtract", default = "default_open_folder_after_extract")]
    pub open_folder_after_extract: bool,
    #[serde(rename = "NestedArchiveDepth", default = "default_nested_depth")]
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

fn default_extractor_type() -> String {
    DEFAULT_EXTRACTOR_TYPE.to_string()
}
fn default_output_encoding() -> String {
    DEFAULT_OUTPUT_ENCODING.to_string()
}
fn default_nested_depth() -> u32 {
    0
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
fn default_delete_source_after_extract() -> bool {
    false
}
fn default_open_folder_after_extract() -> bool {
    false
}

#[derive(Deserialize)]
struct ConfigFile {
    #[serde(rename = "AppSettings")]
    app_settings: AppSettings,
}

pub fn load() -> anyhow::Result<AppSettings> {
    let exe_path = std::env::current_exe()?;
    let default_dir = PathBuf::from(".");
    let base_dir = exe_path.parent().unwrap_or(&default_dir);
    let config_path = base_dir.join("appsettings.json");

    if !config_path.exists() {
        anyhow::bail!("未找到配置文件: {}", config_path.display());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("无法读取配置文件 {}: {}", config_path.display(), e))?;

    let config: ConfigFile =
        serde_json::from_str(&content).map_err(|e| anyhow::anyhow!("配置文件解析失败: {}", e))?;

    let mut settings = config.app_settings;
    
    if !matches!(settings.extractor_type.as_str(), "bandizip" | "7zip") {
        settings.extractor_type = DEFAULT_EXTRACTOR_TYPE.to_string();
    }
    
    if settings.output_encoding.is_empty() {
        settings.output_encoding = DEFAULT_OUTPUT_ENCODING.to_string();
    }
    
    Ok(settings)
}

/// 根据配置创建解压引擎
pub fn create_extractor_from_config(settings: &AppSettings) -> Option<Box<dyn crate::extractor::Extractor>> {
    let path = match settings.extractor_type.as_str() {
        "bandizip" => settings.seven_zip_path.clone(),
        "7zip" => settings.seven_zip_path_7z.clone(),
        _ => return None,
    };

    if path.is_empty() {
        return None;
    }

    crate::extractor::create_extractor(&settings.extractor_type, &path)
}
