#!/bin/sh

# tellme - Uninstallation Script
#
# 卸载 tellme 及其所有配置文件。

set -e

# --- 配置 ---
TELLME_INSTALL_DIR="${TELLME_INSTALL_DIR:-$HOME/.local/bin}"
TELLME_CONFIG_DIR="${TELLME_CONFIG_DIR:-$HOME/.config/tellme}"

ZSHRC_FILE="$HOME/.zshrc"

# --- 颜色定义 ---
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

msg() {
    printf "${GREEN}==>${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}WARN:${NC} %s\n" "$1"
}

msg "开始卸载 tellme..."

# 1. 移除 .zshrc 中的配置
if [ -f "$ZSHRC_FILE" ]; then
    msg "正在从 $ZSHRC_FILE 中移除配置..."
    # 使用 sed 删除相关行，兼容 macOS 和 Linux
    sed -i.bak -e '/tellme\.zsh/d' -e '/# tellme:/d' "$ZSHRC_FILE"
    rm -f "${ZSHRC_FILE}.bak"
fi

# 2. 移除二进制文件
if [ -f "$TELLME_INSTALL_DIR/tellme" ]; then
    msg "正在移除二进制文件: $TELLME_INSTALL_DIR/tellme"
    rm -f "$TELLME_INSTALL_DIR/tellme"
fi

# 3. 移除配置文件和钩子
if [ -d "$TELLME_CONFIG_DIR" ]; then
    msg "正在移除配置文件目录: $TELLME_CONFIG_DIR"
    rm -rf "$TELLME_CONFIG_DIR"
fi

# 4. 清理临时文件
msg "正在清理临时文件..."
rm -f ${TMPDIR:-/tmp}/.tellme_*

msg "tellme 卸载完成。"
warn "请重启你的终端以使更改完全生效。"