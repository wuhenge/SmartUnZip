use serde::{Deserialize, Serialize};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API_URL: &str = "https://api.github.com/repos/wuhenge/SmartUnZip/releases/latest";
const GITHUB_RELEASES_URL: &str = "https://github.com/wuhenge/SmartUnZip/releases";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub download_url: String,
    pub error: Option<String>,
}

pub fn get_current_version() -> String {
    VERSION.to_string()
}

#[allow(dead_code)]
pub fn get_releases_url() -> String {
    GITHUB_RELEASES_URL.to_string()
}

pub fn check_update() -> UpdateInfo {
    let current = get_current_version();

    match fetch_latest_version() {
        Ok(latest) => {
            let has_update = compare_versions(&current, &latest);
            UpdateInfo {
                current_version: current,
                latest_version: latest,
                has_update,
                download_url: GITHUB_RELEASES_URL.to_string(),
                error: None,
            }
        }
        Err(e) => UpdateInfo {
            current_version: current,
            latest_version: String::new(),
            has_update: false,
            download_url: GITHUB_RELEASES_URL.to_string(),
            error: Some(e),
        },
    }
}

fn fetch_latest_version() -> Result<String, String> {
    let response = ureq::get(GITHUB_API_URL)
        .set("User-Agent", &format!("SmartUnZip/{}", get_current_version()))
        .set("Accept", "application/vnd.github.v3+json")
        .call()
        .map_err(|e| format!("网络请求失败: {}", e))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("解析响应失败: {}", e))?;

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
