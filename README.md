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
    ],
    "ExtractNestedArchives": false,
    "NestedArchiveDepth": 1,
    "ExtractNestedFolders": false,
    "AutoExit": false,
    "DebugMode": false,
    "DeleteEmptyFolders": false,
    "CreateFolderThreshold": 1,
    "FlattenWrapperFolder": false
  }
}
```

#### 参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| SevenZipPath | string | - | Bandizip (bz.exe) 的路径 |
| Passwords | array | [] | 尝试解压的密码列表 |
| DeleteFiles | array | [] | 解压后自动删除的文件名（支持模糊匹配） |
| DeleteFolders | array | [] | 解压后自动删除的文件夹名（支持模糊匹配） |
| ExtractNestedArchives | bool | false | 是否解压嵌套压缩包（如zip内包含zip） |
| NestedArchiveDepth | number | 1 | 嵌套压缩包的最大解压层数（上限10） |
| ExtractNestedFolders | bool | false | 是否展平嵌套文件夹（单文件夹时将其内容提升到上级） |
| AutoExit | bool | false | 解压完成后是否自动退出 |
| DebugMode | bool | false | 调试模式，打印命令行参数和压缩包目录结构 |
| DeleteEmptyFolders | bool | false | 解压完成后是否删除空文件夹 |
| CreateFolderThreshold | number | 1 | 当压缩包内无文件夹时，文件数量超过此值则创建以压缩包命名的文件夹（0表示不启用） |
| FlattenWrapperFolder | bool | false | 当解压结果为"单层空文件夹套单文件夹"时，提升内层文件夹到上级并删除空的外层文件夹 |

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