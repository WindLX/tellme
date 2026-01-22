_TELLME_PID="$$"
_TELLME_DIR="${TMPDIR:-/tmp}"
_TELLME_DIR="${_TELLME_DIR%/}"
_TELLME_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/tellme"
_TELLME_STATUS_FILE="$_TELLME_CONFIG_DIR/status"

# 定义文件名
_TELLME_LOG_CURRENT="$_TELLME_DIR/.tellme_current_$_TELLME_PID"
_TELLME_LOG_LAST="$_TELLME_DIR/.tellme_last_$_TELLME_PID"
_TELLME_CMD_CURRENT="$_TELLME_DIR/.tellme_current_cmd_$_TELLME_PID"
_TELLME_CMD_LAST="$_TELLME_DIR/.tellme_last_cmd_$_TELLME_PID"

_tellme_cleanup() {
    rm -f "$_TELLME_LOG_CURRENT" "$_TELLME_LOG_LAST" "$_TELLME_CMD_CURRENT" "$_TELLME_CMD_LAST"
}
add-zsh-hook zshexit _tellme_cleanup

_tellme_preexec() {
    local cmd="$1"
    
    # 如果状态文件存在且内容为 "disabled"，则完全跳过
    if [[ -f "$_TELLME_STATUS_FILE" ]] && [[ "$(cat "$_TELLME_STATUS_FILE")" == "disabled" ]]; then
        return
    fi

    if [[ -f "$_TELLME_LOG_CURRENT" ]]; then
        cp "$_TELLME_LOG_CURRENT" "$_TELLME_LOG_LAST" 2>/dev/null
    fi
    if [[ -f "$_TELLME_CMD_CURRENT" ]]; then
        cp "$_TELLME_CMD_CURRENT" "$_TELLME_CMD_LAST" 2>/dev/null
    fi

    case "$cmd" in
        tellme*|clear*|exit*|cd*|vim*|vi*|nano*|less*|man*|htop*|top*|ssh*|tmux*|source*)
            _TELLME_RECORDING=0
            return
            ;;
    esac

    _TELLME_RECORDING=1
    echo "$cmd" > "$_TELLME_CMD_CURRENT"
    : > "$_TELLME_LOG_CURRENT"
    exec 3>&1 4>&2
    exec > >(tee "$_TELLME_LOG_CURRENT") 2>&1
}

_tellme_precmd() {
    if [[ "$_TELLME_RECORDING" == "1" ]]; then
        exec 1>&3 2>&4 3>&- 4>&-
        _TELLME_RECORDING=0
    fi
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec _tellme_preexec
add-zsh-hook precmd _tellme_precmd

tellme() {
    TELLME_SHELL_PID="$$" \
    TELLME_ROOT="$_TELLME_DIR" \
    TELLME_CONFIG_DIR="$_TELLME_CONFIG_DIR" \
    command tellme "$@"
}