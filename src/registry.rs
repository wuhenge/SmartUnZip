use std::sync::Arc;
use winreg::enums::*;
use winreg::RegKey;

const REG_KEY_PATH: &str = r"Software\Classes\*\shell\SmartUnZip";
const REG_CMD_PATH: &str = r"Software\Classes\*\shell\SmartUnZip\command";

pub fn is_registered() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(REG_KEY_PATH).is_ok()
}

pub fn add(ui: &Arc<crate::ui::ConsoleUi>) {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    match hkcu.create_subkey(REG_KEY_PATH) {
        Ok((shell_key, _)) => {
            if let Err(e) = shell_key.set_value("", &"用 SmartUnZip 解压") {
                ui.error(&format!("设置菜单名称失败: {e}"));
                return;
            }
            if let Err(e) = shell_key.set_value("Icon", &exe_path.as_str()) {
                ui.error(&format!("设置图标失败: {e}"));
                return;
            }
        }
        Err(e) => {
            ui.error(&format!("创建注册表键失败: {e}"));
            return;
        }
    }

    match hkcu.create_subkey(REG_CMD_PATH) {
        Ok((cmd_key, _)) => {
            let cmd = format!("\"{exe_path}\" \"%1\"");
            if let Err(e) = cmd_key.set_value("", &cmd.as_str()) {
                ui.error(&format!("设置命令失败: {e}"));
                return;
            }
        }
        Err(e) => {
            ui.error(&format!("创建命令键失败: {e}"));
            return;
        }
    }

    ui.success("已添加右键菜单");
}

pub fn remove(ui: &Arc<crate::ui::ConsoleUi>) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Err(e) = hkcu.delete_subkey_all(REG_KEY_PATH) {
        ui.error(&format!("移除右键菜单失败: {e}"));
    } else {
        ui.success("已移除右键菜单");
    }
}
