use serde::Deserialize;
use std::path::PathBuf;

const DEFAULT_OUTPUT_ENCODING: &str = if cfg!(windows) { "gbk" } else { "utf-8" };

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub output_encoding: String,
    pub seven_zip_path: String,
    pub output_directory: String,
    pub auto_exit: bool,
    pub extract_nested_folders: bool,
    pub debug_mode: bool,
    pub delete_empty_folders: bool,
    pub flatten_wrapper_folder: bool,
    pub delete_source_after_extract: bool,
    pub open_folder_after_extract: bool,
    pub nested_archive_depth: u32,
    pub create_folder_threshold: u32,
    pub passwords: Vec<String>,
    pub delete_files: Vec<String>,
    pub delete_folders: Vec<String>,
}

#[derive(Deserialize)]
struct RawAppSettings {
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

fn default_create_folder_threshold() -> u32 {
    1
}

impl From<RawAppSettings> for AppSettings {
    fn from(raw: RawAppSettings) -> Self {
        Self {
            output_encoding: if raw.output_encoding.is_empty() {
                DEFAULT_OUTPUT_ENCODING.to_string()
            } else {
                raw.output_encoding
            },
            seven_zip_path: raw.seven_zip_path,
            output_directory: raw.output_directory,
            auto_exit: raw.auto_exit,
            extract_nested_folders: raw.extract_nested_folders,
            debug_mode: raw.debug_mode,
            delete_empty_folders: raw.delete_empty_folders,
            flatten_wrapper_folder: raw.flatten_wrapper_folder,
            delete_source_after_extract: raw.delete_source_after_extract,
            open_folder_after_extract: raw.open_folder_after_extract,
            nested_archive_depth: raw.nested_archive_depth,
            create_folder_threshold: raw.create_folder_threshold,
            passwords: raw.passwords,
            delete_files: raw.delete_files,
            delete_folders: raw.delete_folders,
        }
    }
}

#[derive(Deserialize)]
struct ConfigFile {
    #[serde(rename = "AppSettings")]
    app_settings: RawAppSettings,
}

pub fn load() -> anyhow::Result<AppSettings> {
    let exe_path = std::env::current_exe()?;
    let default_dir = PathBuf::from(".");
    let base_dir = exe_path.parent().unwrap_or(&default_dir);
    let config_path = base_dir.join("appsettings.json");

    if !config_path.exists() {
        anyhow::bail!("未找到配置文件 {}", config_path.display());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("无法读取配置文件 {}: {}", config_path.display(), e))?;

    let config: ConfigFile =
        serde_json::from_str(&content).map_err(|e| anyhow::anyhow!("配置文件解析失败: {}", e))?;

    Ok(config.app_settings.into())
}

pub fn create_extractor_from_config(
    settings: &AppSettings,
) -> Option<Box<dyn crate::extractor::Extractor>> {
    if settings.seven_zip_path.is_empty() {
        return None;
    }

    crate::extractor::create_extractor("7zip", &settings.seven_zip_path)
}
