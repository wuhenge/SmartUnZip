# SmartUnZip

智能解压工具 - 自动尝试密码解压加密压缩包

## 功能特性

- 🚀 自动尝试多个密码解压加密压缩包
- 📊 实时显示解压进度和速度（现代化进度条动画）
- 🖱️ 支持 Windows 右键菜单集成
- 🧹 自动清理临时文件和指定文件/文件夹
- 📦 支持解压嵌套压缩包（如 zip 内包含 zip）
- ⚙️ 可配置的密码列表和解压选项
- 🔄 内置检查更新功能
- 🎨 图形界面配置工具（Tauri GUI）

## 系统要求

- Windows 10/11
- [Bandizip](https://www.bandisoft.com/bandizip/) (bz.exe)
- Rust 1.70+（仅编译时需要）

## 安装

### 下载

从 [Releases](https://github.com/wuhenge/SmartUnZip/releases) 页面下载最新版本。

### 从源码编译

```bash
git clone https://github.com/wuhenge/SmartUnZip.git
cd SmartUnZip
cargo build --release
```

编译后的可执行文件位于 `target/release/smartunzip.exe`

### GUI 配置工具编译

```bash
cd src-tauri
cargo tauri build
```

GUI 配置工具位于 `src-tauri/target/release/smartunzip-gui.exe`

## 配置

首次运行会自动生成 `appsettings.json` 配置文件，也可通过 GUI 工具进行可视化配置：

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
```

### 参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| SevenZipPath | string | - | Bandizip (bz.exe) 的路径 |
| NestedArchiveDepth | number | 0 | 嵌套压缩包的最大解压层数（0 表示不启用，1-10 为解压层数） |
| AutoExit | bool | false | 解压完成后是否自动退出 |
| ExtractNestedFolders | bool | false | 是否展平嵌套文件夹（单文件夹时将其内容提升到上级） |
| DebugMode | bool | false | 调试模式，打印命令行参数和压缩包目录结构 |
| DeleteEmptyFolders | bool | false | 解压完成后是否删除空文件夹 |
| CreateFolderThreshold | number | 1 | 当压缩包内无文件夹时，文件数量超过此值则创建以压缩包命名的文件夹（0 表示不启用） |
| FlattenWrapperFolder | bool | false | 当解压结果为"单层空文件夹套单文件夹"时，提升内层文件夹到上级并删除空的外层文件夹 |
| DeleteSourceAfterExtract | bool | false | 解压完成后是否删除源压缩文件 |
| OpenFolderAfterExtract | bool | false | 解压完成后是否自动打开解压后的文件夹 |
| Passwords | array | [] | 尝试解压的密码列表 |
| DeleteFiles | array | [] | 解压后自动删除的文件名（支持模糊匹配） |
| DeleteFolders | array | [] | 解压后自动删除的文件夹名（支持模糊匹配） |

## 使用方法

### 命令行模式

```bash
# 解压单个文件
smartunzip.exe archive.zip

# 解压多个文件
smartunzip.exe file1.zip file2.rar
```

### GUI 配置工具

运行 `smartunzip-gui.exe` 打开图形界面配置工具：

- 支持深色/浅色主题切换
- 实时验证 Bandizip 路径
- 可视化编辑密码列表和删除规则
- 自动检测配置变更，保存按钮智能启用

### 交互模式

直接运行 `smartunzip.exe` 进入设置界面：

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  SmartUnZip  设置
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ● 右键菜单: 未安装
  ● Bandizip: C:\Program Files\Bandizip\bz.exe

  1. 添加右键菜单
  2. 验证 Bandizip
  3. 检查更新
  0. 退出

  请选择:
```

## 项目结构

```
src/
├── main.rs        # 主程序入口
├── archive.rs     # 压缩包处理逻辑
├── config.rs      # 配置文件管理
├── files.rs       # 文件操作
├── registry.rs    # Windows 注册表操作
└── ui.rs          # 控制台 UI

src-tauri/
├── src/
│   ├── main.rs      # Tauri 主程序
│   ├── lib.rs       # 库入口
│   └── commands.rs  # IPC 命令
├── icons/           # 应用图标
└── tauri.conf.json  # Tauri 配置

ui/
├── index.html       # GUI 界面
├── styles.css       # 样式表
└── main.js          # 前端逻辑
```

## 技术栈

- **后端**: Rust + Tauri 2
- **前端**: HTML5 + CSS3 + JavaScript (Vanilla)
- **UI 设计**: Linear 启发式设计系统
- **解压引擎**: Bandizip (bz.exe)

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 贡献

欢迎提交 Issue 和 Pull Request！

## 致谢

- [Bandizip](https://www.bandisoft.com/bandizip/) - 解压引擎
- [Tauri](https://tauri.app/) - 跨平台应用框架
- [serde](https://serde.rs/) - 序列化框架
- [colored](https://crates.io/crates/colored) - 终端彩色输出
- [ureq](https://docs.rs/ureq/) - HTTP 客户端
