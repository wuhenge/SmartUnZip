#!/bin/bash
set -e

echo "========================================"
echo "SmartUnZip 构建脚本"
echo "========================================"
echo

# 检查 Rust 环境
if ! command -v cargo &> /dev/null; then
    echo "[错误] 未找到 Rust/Cargo 环境，请先安装 Rust"
    exit 1
fi

echo "[1/3] 正在构建 CLI 后端..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "[错误] CLI 构建失败"
    exit 1
fi
echo "[✓] CLI 构建完成: target/release/smartunzip-cli"
echo

echo "[1.5/3] 准备 sidecar 二进制文件..."
# 获取 target triple
TARGET_TRIPLE=$(rustc --print host-tuple)
echo "Target triple: $TARGET_TRIPLE"

# 复制 CLI 到 binaries 目录，添加 target triple 后缀
mkdir -p src-tauri/binaries

# 确定 CLI 二进制文件名
if [ -f target/release/smartunzip-cli.exe ]; then
    CLI_BIN="target/release/smartunzip-cli.exe"
else
    CLI_BIN="target/release/smartunzip-cli"
fi

cp "$CLI_BIN" "src-tauri/binaries/smartunzip-cli-$TARGET_TRIPLE"
echo "[✓] Sidecar 准备完成: src-tauri/binaries/smartunzip-cli-$TARGET_TRIPLE"
echo

echo "[2/3] 正在构建 GUI 前端..."
cd src-tauri

# 检查 tauri-cli
if ! command -v cargo-tauri &> /dev/null; then
    echo "[信息] 正在安装 tauri-cli..."
    cargo install tauri-cli
fi

cargo tauri build --no-bundle
if [ $? -ne 0 ]; then
    echo "[错误] GUI 构建失败"
    cd ..
    exit 1
fi
cd ..
echo "[✓] GUI 构建完成: src-tauri/target/release/smartunzip"
echo

echo "[3/3] 复制构建产物到 dist 目录..."
mkdir -p dist
cp target/release/smartunzip-cli dist/
cp src-tauri/target/release/smartunzip dist/

echo
echo "========================================"
echo "构建完成！"
echo "========================================"
echo "输出文件:"
echo "  - dist/smartunzip-cli  (CLI 后端)"
echo "  - dist/smartunzip      (GUI 前端)"
echo "========================================"
