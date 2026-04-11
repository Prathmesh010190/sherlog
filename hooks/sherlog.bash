# Sherlog shell hook for Bash
# Add the following line to your ~/.bashrc:
#   source <(sherlog init --shell bash)
# Or copy this file and source it directly.

__sherlog_preexec() {
    export __SHERLOG_LAST_CMD="$1"
}

__sherlog_precmd() {
    local exit_code=$?
    if [ $exit_code -ne 0 ] && [ -n "$__SHERLOG_LAST_CMD" ]; then
        local stderr_file="/tmp/sherlog_stderr_$$"
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

# Install the preexec/precmd hooks using PROMPT_COMMAND and trap DEBUG
trap '__sherlog_preexec "$BASH_COMMAND"' DEBUG
PROMPT_COMMAND="__sherlog_precmd;${PROMPT_COMMAND:+$PROMPT_COMMAND}"
