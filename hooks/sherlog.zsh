# Sherlog shell hook for Zsh
# Add the following line to your ~/.zshrc:
#   source <(sherlog init --shell zsh)
# Or copy this file and source it directly.

__sherlog_preexec() {
    export __SHERLOG_LAST_CMD="$1"
    # Capture stderr by redirecting through a temp file
    export __SHERLOG_STDERR_FILE="/tmp/sherlog_stderr_$$"
}

__sherlog_precmd() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]] && [[ -n "$__SHERLOG_LAST_CMD" ]]; then
        local stderr_content=""
        if [[ -f "$__SHERLOG_STDERR_FILE" ]]; then
            stderr_content=$(tail -200 "$__SHERLOG_STDERR_FILE" | sed 's/"/\\"/g' | tr '\n' '\\n' 2>/dev/null || echo "")
            rm -f "$__SHERLOG_STDERR_FILE"
        fi
        cat <<SHERLOG_JSON > /tmp/sherlog_last_error.json
{
  "command": "$(echo "$__SHERLOG_LAST_CMD" | sed 's/"/\\"/g')",
  "exit_code": $exit_code,
  "stderr": "",
  "working_directory": "$(pwd)",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
SHERLOG_JSON
    fi
    unset __SHERLOG_LAST_CMD
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec __sherlog_preexec
add-zsh-hook precmd __sherlog_precmd
