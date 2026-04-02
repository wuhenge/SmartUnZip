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
    ]
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
    ]
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
