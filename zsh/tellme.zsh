# tellme zsh hook

# Called before each command
_tellme_preexec() {
    local cmd="$1"

    # Ask Rust if this command should be captured
    local should_prepare=$(tellme internal --should-prepare "$cmd" 2>/dev/null)
    if [[ "$should_prepare" != "true" ]]; then
        return
    fi

    # Ask Rust to prepare session and get log path
    local log_path=$(tellme internal --prepare "$cmd" 2>/dev/null)

    # Save original stdout/stderr to file descriptors 3 and 4
    exec 3>&1 4>&2

    # Redirect output to the log file via tee
    exec > >(tee "$log_path") 2>&1

    # Set recording flag for precmd
    _TELLME_RECORDING=1
}

# Called after each command
_tellme_precmd() {
    # Restore original stdout/stderr if we were capturing
    if [[ $_TELLME_RECORDING -eq 1 ]]; then
        # Close the tee process and restore original fd
        exec 1>&3 2>&4 3>&- 4>&-
        unset _TELLME_RECORDING
    fi
}

# Cleanup on shell exit
_tellme_cleanup() {
    tellme internal --cleanup 2>/dev/null
}

autoload -Uz add-zsh-hook

add-zsh-hook preexec _tellme_preexec
add-zsh-hook precmd _tellme_precmd
add-zsh-hook zshexit _tellme_cleanup

warn() {
    YELLOW='\033[1;33m'
    NC='\033[0m'
    printf "${YELLOW}WARN:${NC} %s\n" "$1"
}

# Main tellme function - wrapper for the Rust binary
tellme() {
    TELLME_DEV_MODE="${TELLME_DEV_MODE:-0}"
    if [[ $TELLME_DEV_MODE -eq 0 ]]; then
        # Prevent direct calls to internal commands
        local is_internal=0
        for arg in "$@"; do
            if [[ "$arg" == "internal" ]]; then
                is_internal=1
                break
            fi
        done

        if [[ $is_internal -eq 1 ]]; then
            warn "'internal' commands are reserved for system use." >&2
            return 1
        fi

    fi
    
    TELLME_SHELL_PID=$$ \
    command tellme "$@"
}
