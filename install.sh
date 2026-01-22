#!/bin/sh

# tellme - Installation Script
#
# 该脚本会自动安装 tellme 工具及其 Zsh 钩子。
# 支持从 GitHub Releases 下载预编译的二进制文件，或在本地编译。

set -e # 遇到错误立即退出

# --- 配置 ---
REPO="WindLX/tellme" 
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/tellme"
ZSH_HOOK_FILE="$CONFIG_DIR/tellme.zsh"
ZSHRC_FILE="$HOME/.zshrc"

# --- 颜色定义 ---
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# --- 辅助函数 ---
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

# --- 主要逻辑 ---
main() {
    msg "开始安装 tellme..."

    # 1. 检查环境
    if ! command -v zsh >/dev/null; then
        err "Zsh 未安装。tellme 当前仅支持 Zsh。"
    fi
    if ! command -v curl >/dev/null; then
        err "curl 未安装，无法下载所需文件。"
    fi

    # 2. 创建目录
    msg "创建安装目录..."
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$CONFIG_DIR"

    # 3. 获取 tellme 二进制文件
    if ! download_binary; then
        warn "从 GitHub Releases 下载失败。将尝试本地编译..."
        if ! compile_locally; then
            err "本地编译失败。请检查是否已安装 Rust 工具链 (cargo)。"
        fi
    fi

    # 4. 下载 Zsh 钩子脚本
    msg "下载 Zsh 钩子脚本..."
    curl -fsSL "https://raw.githubusercontent.com/$REPO/main/tellme.zsh" -o "$ZSH_HOOK_FILE" || err "下载 Zsh 钩子脚本失败。"
    
    # 5. 配置 .zshrc
    msg "配置 .zshrc..."
    if grep -q "tellme.zsh" "$ZSHRC_FILE"; then
        msg ".zshrc 已配置，跳过。"
    else
        printf "\n# tellme: 捕获上一条命令的输出\n" >> "$ZSHRC_FILE"
        printf "source \"%s\"\n" "$ZSH_HOOK_FILE" >> "$ZSHRC_FILE"
        msg "已将 source 命令添加到 $ZSHRC_FILE。"
        warn "请重启你的终端或运行 'source ~/.zshrc' 来使配置生效。"
    fi
    
    # 6. 检查 PATH
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *) warn "$INSTALL_DIR 不在你的 PATH 环境变量中。请手动添加。" ;;
    esac

    msg "tellme 安装成功！🎉"
}

download_binary() {
    msg "正在尝试从 GitHub Releases 下载预编译的二进制文件..."
    
    # 探测系统架构
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)
    case "$arch" in
        x86_64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            warn "不支持的架构: $arch。无法从 Releases 下载。"
            return 1
            ;;
    esac

    # 获取最新版本 tag
    tag=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$tag" ]; then
        warn "无法获取最新的 Release 版本号。"
        return 1
    fi
    msg "最新版本为: $tag"

    # 构建下载链接
    download_url="https://github.com/$REPO/releases/download/$tag/tellme-${arch}-${os}.tar.gz"
    
    msg "下载链接: $download_url"
    
    # 下载并解压
    if curl -L --fail "$download_url" | tar -xz -C "$INSTALL_DIR" tellme; then
        chmod +x "$INSTALL_DIR/tellme"
        msg "二进制文件下载并安装成功。"
        return 0
    else
        warn "下载或解压失败。"
        return 1
    fi
}

compile_locally() {
    if ! command -v cargo >/dev/null; then
        return 1
    fi
    
    msg "正在从源码编译..."
    
    # 克隆或下载源码
    tmp_dir=$(mktemp -d)
    git clone --depth 1 "https://github.com/$REPO.git" "$tmp_dir"
    
    # 编译
    (cd "$tmp_dir" && cargo build --release) || return 1
    
    # 复制二进制文件
    cp "$tmp_dir/target/release/tellme" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/tellme"
    
    # 清理
    rm -rf "$tmp_dir"
    
    msg "本地编译并安装成功。"
    return 0
}

main