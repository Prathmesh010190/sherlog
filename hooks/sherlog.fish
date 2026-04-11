# Sherlog shell hook for Fish
# Add the following line to your ~/.config/fish/config.fish:
#   sherlog init --shell fish | source
# Or copy this file and source it directly.

function __sherlog_on_exit --on-event fish_postexec
    set -l exit_code $status
    set -l cmd $argv[1]

    if test $exit_code -ne 0
        set -l timestamp (date -u +%Y-%m-%dT%H:%M:%SZ)
        set -l working_dir (pwd)
        set -l escaped_cmd (string replace -a '"' '\\"' -- "$cmd")

        echo '{
  "command": "'$escaped_cmd'",
  "exit_code": '$exit_code',
  "stderr": "",
  "working_directory": "'$working_dir'",
  "timestamp": "'$timestamp'"
}' > /tmp/sherlog_last_error.json
    end
end
