# SmartUnZip

智能解压工具 - 自动尝试密码解压加密压缩包

## 功能特性

- 🚀 自动尝试多个密码解压加密压缩包
- 📊 实时显示解压进度和速度
- 🖱️ 支持Windows右键菜单集成
- 🧹 自动清理临时文件和指定文件/文件夹
- ⚙️ 可配置的密码列表和解压选项

## 系统要求

- Windows 10/11
- [Bandizip](https://www.bandisoft.com/bandizip/) (bz.exe)
- Rust 1.70+

## 安装

### 从源码编译

```bash
git clone https://github.com/yourusername/SmartUnZip.git
cd SmartUnZip
cargo build --release
```

编译后的可执行文件位于 `target/release/smartunzip.exe`

### 配置

首次运行会自动生成 `appsettings.json` 配置文件：

```json
{
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
```

## 使用方法

### 命令行模式

```bash
# 解压单个文件
smartunzip.exe archive.zip

# 解压多个文件
smartunzip.exe file1.zip file2.rar
```

### 交互模式

直接运行 `smartunzip.exe` 进入设置界面：

1. 添加/移除右键菜单
2. 验证Bandizip配置

## 项目结构

```
src/
├── main.rs        # 主程序入口
├── archive.rs     # 压缩包处理逻辑
├── config.rs      # 配置文件管理
├── files.rs       # 文件操作
├── registry.rs    # Windows注册表操作
└── ui.rs          # 控制台UI
```

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 贡献

欢迎提交Issue和Pull Request！

## 致谢

- [Bandizip](https://www.bandisoft.com/bandizip/) - 解压引擎
- [serde](https://serde.rs/) - 序列化框架
- [colored](https://crates.io/crates/colored) - 终端彩色输出