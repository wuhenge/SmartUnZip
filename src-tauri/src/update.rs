use serde::{Deserialize, Serialize};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_RELEASES_URL: &str = "https://github.com/wuhenge/SmartUnZip/releases";
const GITHUB_LATEST_URL: &str = "https://github.com/wuhenge/SmartUnZip/releases/latest";

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
    let response = ureq::get(GITHUB_LATEST_URL)
        .set("User-Agent", &format!("SmartUnZip/{}", get_current_version()))
        .call()
        .map_err(|e| format!("网络请求失败: {}", e))?;

    let body = response
        .into_string()
        .map_err(|e| format!("读取响应失败: {}", e))?;

    parse_version_from_html(&body)
}

fn parse_version_from_html(html: &str) -> Result<String, String> {
    let mut versions = Vec::new();
    let mut remaining = html;
    while let Some(pos) = remaining.find("/releases/tag/") {
        let after = &remaining[pos + 14..];
        if let Some(end) = after.find('"') {
            let tag = &after[..end];
            let version = tag.trim_start_matches('v');
            if !version.is_empty()
                && version
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_digit())
            {
                versions.push(version.to_string());
            }
            remaining = &after[end..];
        } else {
            break;
        }
    }

    versions
        .into_iter()
        .reduce(|a, b| if version_gt(&b, &a) { b } else { a })
        .ok_or_else(|| "无法从页面获取版本信息".to_string())
}

/// 逐段数值比较，判断 a 是否大于 b
fn version_gt(a: &str, b: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|s| s.parse().ok()).collect()
    };
    let a_parts = parse(a);
    let b_parts = parse(b);
    for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
        let a_val = a_parts.get(i).unwrap_or(&0);
        let b_val = b_parts.get(i).unwrap_or(&0);
        if a_val > b_val {
            return true;
        } else if a_val < b_val {
            return false;
        }
    }
    false
}

fn compare_versions(current: &str, latest: &str) -> bool {
    current != latest
}
