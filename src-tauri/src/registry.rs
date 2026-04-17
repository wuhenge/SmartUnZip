#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

const REG_KEY_PATH: &str = r"Software\Classes\*\shell\SmartUnZip";
const REG_CMD_PATH: &str = r"Software\Classes\*\shell\SmartUnZip\command";

#[cfg(windows)]
pub fn is_registered() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(REG_KEY_PATH).is_ok()
}

#[cfg(not(windows))]
pub fn is_registered() -> bool {
    false
}

#[cfg(windows)]
pub fn add() -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("获取程序路径失败: {}", e))?;
    
    let exe_dir = current_exe
        .parent()
        .ok_or("无法获取程序目录")?;
    
    let exe_path = exe_dir.join("smartunzip-cli.exe");
    let exe_path_str = exe_path.to_string_lossy().to_string();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let (shell_key, _) = hkcu
        .create_subkey(REG_KEY_PATH)
        .map_err(|e| format!("创建注册表键失败: {}", e))?;

    shell_key
        .set_value("", &"用 SmartUnZip 解压")
        .map_err(|e| format!("设置菜单名称失败: {}", e))?;

    shell_key
        .set_value("Icon", &exe_path_str.as_str())
        .map_err(|e| format!("设置图标失败: {}", e))?;

    let (cmd_key, _) = hkcu
        .create_subkey(REG_CMD_PATH)
        .map_err(|e| format!("创建命令键失败: {}", e))?;

    let cmd = format!("\"{}\" \"%1\"", exe_path_str);
    cmd_key
        .set_value("", &cmd.as_str())
        .map_err(|e| format!("设置命令失败: {}", e))?;

    Ok(())
}

#[cfg(not(windows))]
pub fn add() -> Result<(), String> {
    Err("右键菜单功能仅支持 Windows".to_string())
}

#[cfg(windows)]
pub fn remove() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.delete_subkey_all(REG_KEY_PATH)
        .map_err(|e| format!("移除右键菜单失败: {}", e))
}

#[cfg(not(windows))]
pub fn remove() -> Result<(), String> {
    Err("右键菜单功能仅支持 Windows".to_string())
}
