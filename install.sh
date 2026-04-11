#!/usr/bin/env bash
set -euo pipefail

# Sherlog installer
# Installs sherlog by downloading a pre-built binary from GitHub releases,
# or building from source if Rust is available and no binary is found.

REPO="sherlog-cli/sherlog"
BINARY_NAME="sherlog"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Colors (disabled if not a terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    NC='\033[0m'
else
    RED='' GREEN='' YELLOW='' CYAN='' BOLD='' NC=''
fi

info()  { echo -e "${CYAN}${BOLD}info:${NC} $*"; }
ok()    { echo -e "${GREEN}${BOLD}  ok:${NC} $*"; }
warn()  { echo -e "${YELLOW}${BOLD}warn:${NC} $*"; }
err()   { echo -e "${RED}${BOLD}error:${NC} $*" >&2; }

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        *)       err "Unsupported OS: $(uname -s)"; exit 1 ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)             err "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac
}

get_latest_version() {
    if command -v curl &>/dev/null; then
        curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | \
            grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
    elif command -v wget &>/dev/null; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | \
            grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
    else
        return 1
    fi
}

download_binary() {
    local os="$1" arch="$2" version="$3"
    local target="${BINARY_NAME}-${version}-${arch}-unknown-${os}-gnu"
    local url="https://github.com/${REPO}/releases/download/${version}/${target}.tar.gz"

    info "Downloading ${BINARY_NAME} ${version} for ${os}/${arch}..."

    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    local archive="${tmpdir}/${target}.tar.gz"

    if command -v curl &>/dev/null; then
        if ! curl -fsSL "$url" -o "$archive" 2>/dev/null; then
            return 1
        fi
    elif command -v wget &>/dev/null; then
        if ! wget -q "$url" -O "$archive" 2>/dev/null; then
            return 1
        fi
    else
        err "Neither curl nor wget found"
        return 1
    fi

    tar -xzf "$archive" -C "$tmpdir"

    # Find the binary in the extracted contents
    local binary
    binary="$(find "$tmpdir" -name "$BINARY_NAME" -type f | head -1)"
    if [ -z "$binary" ]; then
        return 1
    fi

    install_binary "$binary"
}

build_from_source() {
    info "Building from source..."

    if ! command -v cargo &>/dev/null; then
        warn "Rust is not installed. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi

    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    info "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "$tmpdir/sherlog"
    cd "$tmpdir/sherlog"

    info "Compiling (this may take a few minutes)..."
    cargo build --release

    install_binary "$tmpdir/sherlog/target/release/$BINARY_NAME"
}

install_binary() {
    local source="$1"

    chmod +x "$source"

    if [ -w "$INSTALL_DIR" ]; then
        cp "$source" "$INSTALL_DIR/$BINARY_NAME"
    else
        info "Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo cp "$source" "$INSTALL_DIR/$BINARY_NAME"
    fi

    ok "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
}

main() {
    echo ""
    echo -e "${BOLD}Sherlog Installer${NC}"
    echo -e "${CYAN}A developer-focused error debugging assistant${NC}"
    echo ""

    local os arch
    os="$(detect_os)"
    arch="$(detect_arch)"
    info "Detected platform: ${os}/${arch}"

    # Try to download a pre-built binary first
    local version
    version="$(get_latest_version 2>/dev/null || echo "")"

    if [ -n "$version" ]; then
        info "Latest release: ${version}"
        if download_binary "$os" "$arch" "$version"; then
            ok "Installation complete!"
            echo ""
            echo -e "  Get started:"
            echo -e "    ${CYAN}sherlog config set api_key <your-anthropic-key>${NC}"
            echo -e "    ${CYAN}sherlog init${NC}"
            echo ""
            return 0
        fi
        warn "No pre-built binary found for ${os}/${arch}. Falling back to source build."
    else
        warn "Could not fetch latest release. Falling back to source build."
    fi

    # Fall back to building from source
    if ! command -v git &>/dev/null; then
        err "git is required to build from source"
        exit 1
    fi

    build_from_source

    ok "Installation complete!"
    echo ""
    echo -e "  Get started:"
    echo -e "    ${CYAN}sherlog config set api_key <your-anthropic-key>${NC}"
    echo -e "    ${CYAN}sherlog init${NC}"
    echo ""
}

main "$@"
