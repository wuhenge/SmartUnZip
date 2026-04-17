# SmartUnZip

智能解压工具 - 自动尝试密码解压加密压缩包

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## ✨ 功能特性

- 🚀 **智能解压** - 自动尝试多个密码解压加密压缩包
- 🔧 **双引擎支持** - 支持 7-Zip 和 Bandizip 两种解压引擎
- 📊 **实时进度** - 现代化进度条动画，显示解压进度和速度
- 🖱️ **右键集成** - Windows 右键菜单一键解压
- 🧹 **自动清理** - 解压后自动删除临时文件和指定文件/文件夹
- 📦 **嵌套解压** - 支持解压嵌套压缩包（如 zip 内包含 zip）
- 🌐 **编码支持** - 可配置输出编码（GBK/UTF-8/Shift_JIS 等），解决中文乱码
- 📂 **自定义输出** - 可配置解压输出目录，不指定则解压到压缩包所在目录
- 🖥️ **跨平台** - 支持 Windows、macOS、Linux
- ⚙️ **灵活配置** - 可配置的密码列表和解压选项
- 🔄 **检查更新** - 内置检查更新功能（GUI 模式）
- 🎨 **图形界面** - Tauri 驱动的现代化配置界面

## 📋 系统要求

| 平台 | 要求 |
|------|------|
| Windows | Windows 10/11 |
| macOS | macOS 10.15+ |
| Linux | Ubuntu 22.04+ 或其他主流发行版 |

- [7-Zip](https://7-zip.org/) (7z/7z.exe) 或 [Bandizip](https://www.bandisoft.com/bandizip/) (bz.exe)（仅 Windows）
- Rust 1.70+（仅编译时需要）

> ⚠️ **注意**：本工具调用 7-Zip 或 Bandizip 的命令行接口。Bandizip 免费版仅供个人使用，在商业环境或企业内部使用请自行向 Bandisoft 购买合适的许可证。macOS/Linux 用户推荐安装 [7-Zip](https://7-zip.org/)（Linux 下为 `p7zip` 或 `7zz`）。

## 📥 安装

### 下载预编译版本

从 [Releases](https://github.com/wuhenge/SmartUnZip/releases) 页面下载最新版本。

| 平台 | 文件 |
|------|------|
| Windows | `*_x64_zh-CN_setup.exe`（安装版）或 `*_portable.zip`（绿色版） |
| macOS (Apple Silicon) | `*_aarch64_macOS_*.dmg` |
| macOS (Intel) | `*_x86_64_macOS_*.dmg` |
| Linux | `*_x64_linux_*.deb` 或 `*.AppImage` |

### 从源码编译

```bash
git clone https://github.com/wuhenge/SmartUnZip.git
cd SmartUnZip

# 构建 CLI
cargo build --release -p smartunzip-cli

# 构建 GUI（需先安装 tauri-cli）
cargo install tauri-cli
cd src-tauri
cargo tauri build
```

构建产物：
- CLI: `target/release/smartunzip-cli`（Windows 为 `.exe`）
- GUI: `src-tauri/target/release/bundle/` 下各平台安装包

## ⚙️ 配置

配置文件为可执行文件同目录下的 `appsettings.json`，可通过 GUI 工具进行可视化配置。

### 配置文件示例

```json
{
  "AppSettings": {
    "ExtractorType": "7zip",
    "SevenZipPath": "C:\\Program Files\\Bandizip\\bz.exe",
    "SevenZipPath7z": "C:\\Program Files\\7-Zip\\7z.exe",
    "OutputEncoding": "gbk",
    "OutputDirectory": "",
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
| `ExtractorType` | string | `"7zip"` | 解压引擎，可选 `"7zip"` 或 `"bandizip"` |
| `SevenZipPath` | string | - | Bandizip (bz.exe) 的路径 |
| `SevenZipPath7z` | string | - | 7-Zip (7z/7z.exe/7zz) 的路径 |
| `OutputEncoding` | string | 见右 | 输出编码，Windows: `gbk`，其他: `utf-8`，还支持 `shift_jis`、`euc-kr`、`big5` |
| `OutputDirectory` | string | `""` | 自定义解压输出目录，为空则解压到压缩包所在目录 |
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
smartunzip-cli archive.zip

# 解压多个文件
smartunzip-cli file1.zip file2.rar file3.7z
```

### 无参数运行

直接运行 `smartunzip-cli`（不带参数）将验证配置文件是否有效：

- 配置有效时显示解压引擎和路径信息
- 配置无效时提示具体原因（未配置路径、文件不存在等）
- 配置文件不存在时提示未找到配置文件

### GUI 配置工具

运行 `smartunzip` 打开图形界面：

- 🌓 深色/浅色主题切换
- ✅ 实时验证解压引擎路径
- 📝 可视化编辑密码列表和删除规则
- 📂 可选择自定义解压输出目录
- 💾 自动检测配置变更
- 🖱️ Windows 右键菜单集成

## 📁 项目结构

```
SmartUnZip/
├── cli/                    # CLI 后端（smartunzip-cli）
│   └── src/
│       ├── main.rs         # 主程序入口
│       ├── archive.rs      # 压缩包处理逻辑
│       ├── config.rs       # 配置文件管理
│       ├── files.rs        # 文件操作
│       ├── registry.rs     # Windows 注册表操作
│       ├── ui.rs           # 控制台 UI
│       └── extractor/      # 解压引擎抽象
│           ├── mod.rs       # Extractor trait 定义
│           ├── sevenzip.rs  # 7-Zip 引擎实现
│           └── bandizip.rs  # Bandizip 引擎实现
├── src-tauri/              # GUI 前端（smartunzip）
│   ├── src/
│   │   ├── main.rs         # Tauri 主程序
│   │   ├── lib.rs          # 库入口
│   │   ├── commands.rs     # IPC 命令
│   │   ├── registry.rs     # Windows 注册表操作
│   │   └── update.rs       # 更新检查
│   ├── capabilities/       # Tauri 权限配置
│   ├── icons/              # 应用图标
│   └── tauri.conf.json     # Tauri 配置
├── ui/                     # 前端界面
│   ├── index.html
│   ├── styles.css
│   └── main.js
├── .cargo/config.toml      # Cargo 编译优化配置
├── .github/workflows/      # GitHub Actions 构建工作流
├── Cargo.toml              # Workspace 根配置
└── README.md
```

## 🛠️ 技术栈

- **后端**: Rust + Tauri 2
- **前端**: HTML5 + CSS3 + JavaScript (Vanilla)
- **解压引擎**: 7-Zip / Bandizip
- **编码处理**: [encoding_rs](https://crates.io/crates/encoding_rs)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目采用 [MIT](LICENSE) 许可证。
