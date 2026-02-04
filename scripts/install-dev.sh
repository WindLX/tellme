#!/usr/bin/env zsh

# tellme-dev - Installation Script (Dev)
#
# 该脚本会自动从本地源码编译 tellme。
# 适用于开发测试
#
# 重构后架构:
# - Rust 二进制处理状态管理、命令过滤、文件管理
# - Zsh 钩子仅负责调用 Rust 并重定向输出

set -e # 遇到错误立即退出

# --- 配置 ---
SCRIPT_DIR="$(dirname "$(realpath "$0")")"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TARGET_BIN="$PROJECT_ROOT/target/debug/tellme"
TELLME_ZSH_SRC="$PROJECT_ROOT/zsh/tellme.zsh"
ZSHRC_FILE="$HOME/.zshrc"

TELLME_DEV_DIR="$PROJECT_ROOT/tmp"
TELLME_CONFIG_DIR="$TELLME_DEV_DIR/config-dev"
TELLME_TEMP_DIR="$TELLME_DEV_DIR/temp-dev"

# --- 颜色定义 ---
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

msg() {
    printf "${GREEN}==>${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}WARN:${NC} %s\n" "$1"
}

err() {
    printf "${RED}ERROR:${NC} %s\n" "$1" >&2
    exit 1
}

main() {
    msg "启动 Tellme 开发环境..."

    # 1. 编译
    msg "编译 tellme 二进制文件..."
    cd "$PROJECT_ROOT" && cargo build

    # 2. 准备目录
    msg "准备配置和临时目录..."
    mkdir -p "$TELLME_CONFIG_DIR"
    mkdir -p "$TELLME_TEMP_DIR"
    local TEMP_ZDOTDIR=$(mktemp -d)
    
    # 3. 准备 .zshrc
    msg "配置临时 .zshrc..."
    cat <<EOF > "$TEMP_ZDOTDIR/.zshrc"
# 1. 尝试加载用户的默认配置
if [[ -f "$ZSHRC_FILE" ]]; then
    source "$ZSHRC_FILE"
fi

# 2. 设置环境变量
export PATH="$PROJECT_ROOT/target/debug:\$PATH"
export TELLME_CONFIG_DIR="$TELLME_CONFIG_DIR"
export TELLME_TEMP_DIR="$TELLME_TEMP_DIR"

# 3. 加载插件
source "$TELLME_ZSH_SRC"

# 4. 视觉提示
PROMPT="%F{yellow}[tellme-dev]%f \$PROMPT"
echo "${GREEN}>>> Tellme 开发环境 (PID: \$$) <<<${NC}"
EOF

    msg "启动沙箱 Shell..."
    msg "输入 'exit' 退出并清理。"

    # 4. 启动 Shell
    env ZDOTDIR="$TEMP_ZDOTDIR" zsh

    # 5. 清理
    msg "正在清理临时文件..."
    rm -rf "$TEMP_ZDOTDIR"
    rm -rf "$TELLME_DEV_DIR"
    
    msg "已退出开发环境。👋"
}

main
