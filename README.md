# SmartUnZip

智能解压工具 - 自动尝试密码解压加密压缩包

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## ✨ 功能特性

- 🚀 **智能解压** - 自动尝试多个密码解压加密压缩包
- 📊 **实时进度** - 现代化进度条动画，显示解压进度和速度
- 🖱️ **右键集成** - Windows 右键菜单一键解压
- 🧹 **自动清理** - 解压后自动删除临时文件和指定文件/文件夹
- 📦 **嵌套解压** - 支持解压嵌套压缩包（如 zip 内包含 zip）
- ⚙️ **灵活配置** - 可配置的密码列表和解压选项
- 🔄 **检查更新** - 内置检查更新功能（GUI 模式）
- 🎨 **图形界面** - Tauri 驱动的现代化配置界面

## 📋 系统要求

- Windows 10/11
- [Bandizip](https://www.bandisoft.com/bandizip/) (bz.exe)
- Rust 1.70+（仅编译时需要）

> ⚠️ **注意**：本工具仅为方便调用 Bandizip 的免费版。根据 Bandizip 官方政策，其免费版仅供个人使用，在商业环境或企业内部使用请自行向 Bandisoft 购买合适的许可证。

## 📥 安装

### 方式一：下载预编译版本

从 [Releases](https://github.com/wuhenge/SmartUnZip/releases) 页面下载最新版本。

### 方式二：从源码编译

```bash
# 克隆仓库
git clone https://github.com/wuhenge/SmartUnZip.git
cd SmartUnZip

# 一键构建（推荐）
build.bat

# 或手动构建
# CLI 后端
cargo build --release

# GUI 前端
cd src-tauri
cargo tauri build
```

构建产物：
- CLI: `target/release/smartunzip-cli.exe`
- GUI: `src-tauri/target/release/smartunzip.exe`

## ⚙️ 配置

首次运行会自动生成 `appsettings.json` 配置文件，也可通过 GUI 工具进行可视化配置。

### 配置文件示例

```json
{
  "AppSettings": {
    "SevenZipPath": "C:\\Program Files\\Bandizip\\bz.exe",
    "NestedArchiveDepth": 0,
    "AutoExit": false,
    "ExtractNestedFolders": false,
    "DebugMode": false,
    "DeleteEmptyFolders": false,
    "CreateFolderThreshold": 1,
    "FlattenWrapperFolder": false,
    "DeleteSourceAfterExtract": false,
    "OpenFolderAfterExtract": false,
    "Passwords": ["1234", "www", "1111"],
    "DeleteFiles": ["说明.txt", "更多资源.url"],
    "DeleteFolders": ["说明"]
  }
}
```

### 参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `SevenZipPath` | string | - | Bandizip (bz.exe) 的路径 |
| `NestedArchiveDepth` | number | 0 | 嵌套压缩包最大解压层数（0=禁用） |
| `AutoExit` | bool | false | 解压完成后自动退出 |
| `ExtractNestedFolders` | bool | false | 展平嵌套文件夹 |
| `DebugMode` | bool | false | 调试模式，打印详细信息 |
| `DeleteEmptyFolders` | bool | false | 删除空文件夹 |
| `CreateFolderThreshold` | number | 1 | 文件数超过此值时创建文件夹（0=禁用） |
| `FlattenWrapperFolder` | bool | false | 提升内层文件夹到上级 |
| `DeleteSourceAfterExtract` | bool | false | 解压后删除源压缩文件 |
| `OpenFolderAfterExtract` | bool | false | 解压后自动打开文件夹 |
| `Passwords` | array | [] | 尝试解压的密码列表 |
| `DeleteFiles` | array | [] | 解压后自动删除的文件名 |
| `DeleteFolders` | array | [] | 解压后自动删除的文件夹名 |

## 🚀 使用方法

### 命令行模式

```bash
# 解压单个文件
smartunzip-cli.exe archive.zip

# 解压多个文件
smartunzip-cli.exe file1.zip file2.rar file3.7z
```

### GUI 配置工具

运行 `smartunzip.exe` 打开图形界面：

- 🌓 深色/浅色主题切换
- ✅ 实时验证 Bandizip 路径
- 📝 可视化编辑密码列表和删除规则
- 💾 自动检测配置变更

### 交互模式

直接运行 `smartunzip-cli.exe`（不带参数）进入设置界面：

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  SmartUnZip  设置
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ● 右键菜单: 未安装
  ● Bandizip: C:\Program Files\Bandizip\bz.exe

  1. 添加右键菜单
  2. 验证 Bandizip
  0. 退出

  请选择:
```

## 📁 项目结构

```
SmartUnZip/
├── src/                    # CLI 后端源码
│   ├── main.rs            # 主程序入口
│   ├── archive.rs         # 压缩包处理逻辑
│   ├── config.rs          # 配置文件管理
│   ├── files.rs           # 文件操作
│   ├── registry.rs        # Windows 注册表操作
│   └── ui.rs              # 控制台 UI
├── src-tauri/             # GUI 前端（Tauri）
│   ├── src/
│   │   ├── main.rs        # Tauri 主程序
│   │   ├── lib.rs         # 库入口
│   │   ├── commands.rs    # IPC 命令
│   │   └── update.rs      # 更新检查
│   ├── binaries/          # Sidecar 二进制文件
│   ├── icons/             # 应用图标
│   └── tauri.conf.json    # Tauri 配置
├── ui/                    # 前端界面
│   ├── index.html         # GUI 界面
│   ├── styles.css         # 样式表
│   └── main.js            # 前端逻辑
├── build.bat              # 一键构建脚本
├── Cargo.toml             # Rust 配置
└── README.md              # 本文件
```

## 🛠️ 技术栈

- **后端**: Rust + Tauri 2
- **前端**: HTML5 + CSS3 + JavaScript (Vanilla)
- **UI 设计**: Linear 启发式设计系统
- **解压引擎**: Bandizip (bz.exe)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目采用 [MIT](LICENSE) 许可证。

## 🙏 致谢

- [Bandizip](https://www.bandisoft.com/bandizip/) - 解压引擎
- [Tauri](https://tauri.app/) - 跨平台应用框架
- [serde](https://serde.rs/) - 序列化框架
- [colored](https://crates.io/crates/colored) - 终端彩色输出
