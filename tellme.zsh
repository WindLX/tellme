# 使用 Shell 的 PID 区分不同终端窗口
_TELLME_PID="$$"
# 临时目录路径
_TELLME_DIR="${TMPDIR:-/tmp}"
# 移除结尾的斜杠
_TELLME_DIR="${_TELLME_DIR%/}"

# 定义文件名
_TELLME_LOG_CURRENT="$_TELLME_DIR/.tellme_current_$_TELLME_PID"
_TELLME_LOG_LAST="$_TELLME_DIR/.tellme_last_$_TELLME_PID"
_TELLME_CMD_CURRENT="$_TELLME_DIR/.tellme_current_cmd_$_TELLME_PID"
_TELLME_CMD_LAST="$_TELLME_DIR/.tellme_last_cmd_$_TELLME_PID"

# 清理函数
_tellme_cleanup() {
    rm -f "$_TELLME_LOG_CURRENT" "$_TELLME_LOG_LAST" "$_TELLME_CMD_CURRENT" "$_TELLME_CMD_LAST"
}
add-zsh-hook zshexit _tellme_cleanup

_tellme_preexec() {
    local cmd="$1"
    
    # ====================================================
    # 1. 轮转逻辑必须最先执行
    # ====================================================
    # 只有当 Current 文件存在，且当前命令不是 tellme 自身导致的连环调用时，才进行轮转
    # 注意：我们希望 tellme 运行时，能把上一条命令（Current 里的）归档到 Last
    if [[ -f "$_TELLME_LOG_CURRENT" ]]; then
        # Current 里有东西时，我们尝试备份，把它视为上一条命令的遗产，移入 Last
        cp "$_TELLME_LOG_CURRENT" "$_TELLME_LOG_LAST" 2>/dev/null
    fi
    if [[ -f "$_TELLME_CMD_CURRENT" ]]; then
        cp "$_TELLME_CMD_CURRENT" "$_TELLME_CMD_LAST" 2>/dev/null
    fi

    # ====================================================
    # 2. 排除逻辑：决定本条命令是否录制
    # ====================================================
    case "$cmd" in
        tellme*|clear*|exit*|cd*|vim*|vi*|nano*|less*|man*|htop*|btop*|top*|ssh*|tmux*|source*)
            _TELLME_RECORDING=0
            return
            ;;
    esac

    # ====================================================
    # 3. 录制逻辑
    # ====================================================
    _TELLME_RECORDING=1
    
    # 写入当前命令名
    echo "$cmd" > "$_TELLME_CMD_CURRENT"
    # 创建/清空 Current Log 文件
    : > "$_TELLME_LOG_CURRENT"
    
    # 魔法重定向
    exec 3>&1 4>&2
    exec > >(tee "$_TELLME_LOG_CURRENT") 2>&1
}

_tellme_precmd() {
    # 只有处于录制状态才需要恢复 FD
    if [[ "$_TELLME_RECORDING" == "1" ]]; then
        exec 1>&3 2>&4 3>&- 4>&-
        _TELLME_RECORDING=0
    fi
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec _tellme_preexec
add-zsh-hook precmd _tellme_precmd

tellme() {
    # 传递 PID 和 确定的目录路径 给 Rust
    TELLME_SHELL_PID="$$" \
    TELLME_ROOT="$_TELLME_DIR" \
    command tellme "$@"
}