use std::sync::Arc;

const REG_KEY_PATH: &str = r"Software\Classes\*\shell\SmartUnZip";
const REG_CMD_PATH: &str = r"Software\Classes\*\shell\SmartUnZip\command";

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

pub fn is_registered() -> bool {
    #[cfg(windows)]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey(REG_KEY_PATH).is_ok()
    }

    #[cfg(not(windows))]
    {
        false
    }
}

#[cfg(windows)]
fn add_internal() -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("获取程序路径失败: {}", e))?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let (shell_key, _) = hkcu
        .create_subkey(REG_KEY_PATH)
        .map_err(|e| format!("创建注册表键失败: {}", e))?;

    shell_key
        .set_value("", &"用 SmartUnZip 解压")
        .map_err(|e| format!("设置菜单名称失败: {}", e))?;

    shell_key
        .set_value("Icon", &exe_path.as_str())
        .map_err(|e| format!("设置图标失败: {}", e))?;

    let (cmd_key, _) = hkcu
        .create_subkey(REG_CMD_PATH)
        .map_err(|e| format!("创建命令键失败: {}", e))?;

    let cmd = format!("\"{}\" \"%1\"", exe_path);
    cmd_key
        .set_value("", &cmd.as_str())
        .map_err(|e| format!("设置命令失败: {}", e))?;

    Ok(())
}

#[cfg(windows)]
fn remove_internal() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.delete_subkey_all(REG_KEY_PATH)
        .map_err(|e| format!("移除右键菜单失败: {}", e))
}

pub fn add(ui: &Arc<crate::ui::ConsoleUi>) {
    #[cfg(windows)]
    {
        match add_internal() {
            Ok(()) => ui.success("已添加右键菜单"),
            Err(e) => ui.error(&e),
        }
    }

    #[cfg(not(windows))]
    {
        ui.warn("右键菜单功能仅支持 Windows");
    }
}

pub fn remove(ui: &Arc<crate::ui::ConsoleUi>) {
    #[cfg(windows)]
    {
        match remove_internal() {
            Ok(()) => ui.success("已移除右键菜单"),
            Err(e) => ui.error(&e),
        }
    }

    #[cfg(not(windows))]
    {
        ui.warn("右键菜单功能仅支持 Windows");
    }
}
