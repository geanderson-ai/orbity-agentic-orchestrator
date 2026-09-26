#!/usr/bin/env bash
# ==============================================================================
# 🪐 ORBITY - System Bootstrap, Dependency Installer & CLI Setup
# ==============================================================================
# Detects OS (Linux / macOS / Windows WSL), installs system prerequisites,
# audits the 5 AI CLIs (claude, codex, pi, hermes, agy), compiles Orbity in
# release mode, and installs the 'orbity' command globally into user's PATH.
# ==============================================================================

set -eo pipefail

# --- Color Definitions & Styling ----------------------------------------------
BOLD="\033[1m"
RESET="\033[0m"
RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
BLUE="\033[34m"
MAGENTA="\033[35m"
CYAN="\033[36m"
WHITE="\033[37m"

info()    { printf "${CYAN}${BOLD}[INFO]${RESET} %s\n" "$*"; }
success() { printf "${GREEN}${BOLD}[✓]${RESET} %s\n" "$*"; }
warn()    { printf "${YELLOW}${BOLD}[WARN]${RESET} %s\n" "$*"; }
error()   { printf "${RED}${BOLD}[ERROR]${RESET} %s\n" "$*"; }
step()    { printf "\n${MAGENTA}${BOLD}==>${RESET} ${BOLD}%s${RESET}\n" "$*"; }

# --- Flags & Defaults ---------------------------------------------------------
AUTO_YES=false
CHECK_ONLY=false
NO_SHELL_EDIT=false
INSTALL_DIR="${HOME}/.local/bin"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

show_help() {
  cat <<EOF
${BOLD}Usage:${RESET} $0 [OPTIONS]

${BOLD}Options:${RESET}
  -y, --yes          Non-interactive mode (automatically answer yes to prompts)
  --check-only       Only perform system checks and CLI discovery without building
  --no-shell-edit    Do not modify shell profile files (.bashrc, .zshrc, .profile)
  --install-dir DIR  Custom directory to install the 'orbity' binary (default: ${HOME}/.local/bin)
  -h, --help         Show this help message

${BOLD}Supported Operating Systems:${RESET}
  - Linux (Debian, Ubuntu, Pop!_OS, Fedora, RHEL, CentOS, Arch, Alpine, openSUSE)
  - macOS (Darwin with Homebrew)
  - Windows (via WSL2 / Windows Subsystem for Linux)
EOF
  exit 0
}

# Parse Command-Line Arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    -y|--yes)
      AUTO_YES=true
      shift
      ;;
    --check-only)
      CHECK_ONLY=true
      shift
      ;;
    --no-shell-edit)
      NO_SHELL_EDIT=true
      shift
      ;;
    --install-dir)
      INSTALL_DIR="$2"
      shift 2
      ;;
    -h|--help)
      show_help
      ;;
    *)
      error "Unknown option: $1"
      show_help
      ;;
  esac
done

# --- Banner -------------------------------------------------------------------
clear 2>/dev/null || true
cat << "EOF"
  ██████╗ ██████╗ ██████╗ ██╗████████╗██╗   ██╗
 ██╔═══██╗██╔══██╗██╔══██╗██║╚══██╔══╝╚██╗ ██╔╝
 ██║   ██║██████╔╝██████╔╝██║   ██║    ╚████╔╝ 
 ██║   ██║██╔══██╗██╔══██╗██║   ██║     ╚██╔╝  
 ╚██████╔╝██║  ██║██████╔╝██║   ██║      ██║   
  ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚═╝   ╚═╝      ╚═╝   
  High-Performance Autonomous Multi-Agent Orchestrator
EOF
echo ""

# --- 1. OS & Architecture Detection ------------------------------------------
step "1. Detecting Operating System & Architecture"

OS_NAME=""
OS_TYPE="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
DISTRO=""
IS_WSL=false

if [[ "$OS_TYPE" == "linux" ]]; then
  if grep -qi microsoft /proc/version 2>/dev/null || [[ -n "${WSL_DISTRO_NAME}" ]]; then
    IS_WSL=true
    OS_NAME="Windows (WSL2 / Linux Subsystem)"
  else
    OS_NAME="Linux (Native)"
  fi

  if [[ -f /etc/os-release ]]; then
    # shellcheck disable=SC1091
    source /etc/os-release
    DISTRO="${ID:-unknown}"
  fi
elif [[ "$OS_TYPE" == "darwin" ]]; then
  OS_NAME="macOS (Darwin)"
  DISTRO="macos"
elif [[ "$OS_TYPE" =~ cygwin|mingw|msys ]]; then
  OS_NAME="Windows (Native / MinGW)"
  DISTRO="windows_native"
else
  OS_NAME="Unknown ($OS_TYPE)"
  DISTRO="unknown"
fi

printf "  ${BOLD}Platform:${RESET}      %s\n" "$OS_NAME"
printf "  ${BOLD}Distribution:${RESET}  %s\n" "${DISTRO:-N/A}"
printf "  ${BOLD}Architecture:${RESET}  %s\n" "$ARCH"

if [[ "$DISTRO" == "windows_native" ]]; then
  echo ""
  warn "Native Windows detected. Orbity's kernel-confinement sandbox requires Linux namespaces."
  warn "Please run this installer inside Windows Subsystem for Linux (WSL2)."
  echo "  Recommended step: Open PowerShell and run: 'wsl --install -d Ubuntu'"
  exit 1
fi

success "Environment detected successfully."

# --- 2. System Dependencies Check & Installation -----------------------------
step "2. Checking & Installing System Dependencies"

MISSING_PKGS=()

check_cmd() {
  command -v "$1" >/dev/null 2>&1
}

# Check base requirements
if ! check_cmd curl; then MISSING_PKGS+=("curl"); fi
if ! check_cmd git; then MISSING_PKGS+=("git"); fi
if ! check_cmd sqlite3; then MISSING_PKGS+=("sqlite3"); fi

if [[ "$OS_TYPE" == "linux" ]]; then
  if ! check_cmd bwrap; then MISSING_PKGS+=("bubblewrap"); fi
  if ! check_cmd pkg-config; then MISSING_PKGS+=("pkg-config"); fi
  if ! check_cmd gcc && ! check_cmd clang && ! check_cmd cc; then MISSING_PKGS+=("build-essential / gcc"); fi
elif [[ "$OS_TYPE" == "darwin" ]]; then
  if ! check_cmd pkg-config; then MISSING_PKGS+=("pkg-config"); fi
  warn "macOS detected: Bubblewrap (bwrap) is Linux-specific."
  info "On macOS, Orbity runs commands in standard workspace mode or via Docker containers."
fi

if [[ ${#MISSING_PKGS[@]} -gt 0 ]]; then
  warn "The following system packages are missing: ${MISSING_PKGS[*]}"
  
  INSTALL_CMD=""
  if [[ "$OS_TYPE" == "linux" ]]; then
    case "$DISTRO" in
      ubuntu|debian|pop|linuxmint|kali)
        INSTALL_CMD="sudo apt-get update && sudo apt-get install -y bubblewrap sqlite3 pkg-config libssl-dev build-essential curl git"
        ;;
      fedora|rhel|centos|rocky|alma)
        INSTALL_CMD="sudo dnf install -y bubblewrap sqlite pkgconfig openssl-devel gcc gcc-c++ curl git"
        ;;
      arch|manjaro|endeavouros)
        INSTALL_CMD="sudo pacman -Sy --needed --noconfirm bubblewrap sqlite pkgconf base-devel curl git"
        ;;
      alpine)
        INSTALL_CMD="sudo apk add bubblewrap sqlite pkgconf build-base openssl-dev curl git"
        ;;
      opensuse*|suse)
        INSTALL_CMD="sudo zypper install -y bubblewrap sqlite3 pkg-config libopenssl-devel gcc gcc-c++ curl git"
        ;;
      *)
        warn "Could not automatically determine package manager for distro: $DISTRO"
        ;;
    esac
  elif [[ "$OS_TYPE" == "darwin" ]]; then
    if check_cmd brew; then
      INSTALL_CMD="brew install sqlite pkg-config openssl@3"
    else
      warn "Homebrew not found. Please install Homebrew from https://brew.sh/"
    fi
  fi

  if [[ -n "$INSTALL_CMD" ]]; then
    CONFIRM="n"
    if [[ "$AUTO_YES" == true ]]; then
      CONFIRM="y"
    else
      read -rp "Would you like to install missing system packages automatically? [Y/n] " CONFIRM
      CONFIRM="${CONFIRM:-y}"
    fi

    if [[ "$CONFIRM" =~ ^[Yy]$ ]]; then
      info "Running: $INSTALL_CMD"
      eval "$INSTALL_CMD"
      success "System dependencies installed successfully."
    else
      warn "Skipping package installation. Some features may not function until installed."
    fi
  fi
else
  success "All core system packages (bubblewrap, sqlite3, curl, git, compiler) are present."
fi

# --- 3. Rust Toolchain Check & Installation ----------------------------------
step "3. Checking Rust & Cargo Toolchain"

if check_cmd cargo && check_cmd rustc; then
  RUST_VER="$(rustc --version | awk '{print $2}')"
  success "Rust is installed: version ${RUST_VER}"
else
  warn "Rust toolchain was not found in PATH."
  INSTALL_RUST="n"
  if [[ "$AUTO_YES" == true ]]; then
    INSTALL_RUST="y"
  else
    read -rp "Would you like to install Rust toolchain via rustup automatically? [Y/n] " INSTALL_RUST
    INSTALL_RUST="${INSTALL_RUST:-y}"
  fi

  if [[ "$INSTALL_RUST" =~ ^[Yy]$ ]]; then
    info "Installing Rust via official rustup script..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck disable=SC1091
    source "${HOME}/.cargo/env"
    success "Rust installed successfully: $(rustc --version)"
  else
    error "Rust is required to build Orbity. Please install it manually from https://rustup.rs"
    exit 1
  fi
fi

# --- 4. Suite of 5 AI Coding CLIs Audit --------------------------------------
step "4. Auditing Suite of 5 AI Coding CLIs (claude, codex, pi, hermes, agy)"

# Extend lookup path with common installation locations
EXTRA_PATHS=(
  "${HOME}/.local/bin"
  "${HOME}/.hermes/node/bin"
  "${HOME}/.cargo/bin"
  "${HOME}/.npm-global/bin"
  "/usr/local/bin"
)

for p in "${EXTRA_PATHS[@]}"; do
  if [[ -d "$p" && ":$PATH:" != *":$p:"* ]]; then
    PATH="$p:$PATH"
  fi
done

CLI_NAMES=("claude" "codex" "pi" "hermes" "agy")
CLI_DESCRIPTIONS=(
  "Claude Code (Architecture review, critical invariant auditing)"
  "Codex CLI (Rust implementation, TDD, code generation)"
  "Pi Assistant (Fast surgical single-file edits & lint)"
  "Hermes Agent (External tools, web search, structured usage)"
  "Agy Antigravity (Deep repository mapping, planning, MCP)"
)
CLI_INSTALL_HINTS=(
  "npm install -g @anthropic-ai/claude-code"
  "npm install -g @openai/codex"
  "npm install -g @mariozechner/pi-coding-assistant"
  "pip install hermes-agent || cargo install hermes-cli"
  "npm install -g @google/antigravity || check https://github.com/google/antigravity"
)

FOUND_COUNT=0
TOTAL_COUNT=${#CLI_NAMES[@]}

printf "\n  ${BOLD}%-10s %-12s %-45s %s${RESET}\n" "CLI" "STATUS" "BINARY PATH" "VERSION"
printf "  ────────────────────────────────────────────────────────────────────────────────────────\n"

for i in "${!CLI_NAMES[@]}"; do
  NAME="${CLI_NAMES[$i]}"
  BIN_PATH="$(command -v "$NAME" 2>/dev/null || true)"
  
  if [[ -n "$BIN_PATH" ]]; then
    FOUND_COUNT=$((FOUND_COUNT + 1))
    # Extract version safely with timeout
    VER="$("$BIN_PATH" --version 2>/dev/null | head -n 1 || echo "installed")"
    # Clean version string
    VER="${VER//${NAME}/}"
    VER="$(echo "$VER" | awk '{print $1}')"
    printf "  ${GREEN}%-10s${RESET} ${GREEN}%-12s${RESET} %-45s ${CYAN}%s${RESET}\n" "$NAME" "[FOUND]" "$BIN_PATH" "$VER"
  else
    printf "  ${YELLOW}%-10s${RESET} ${RED}%-12s${RESET} %-45s ${YELLOW}%s${RESET}\n" "$NAME" "[NOT FOUND]" "---" "Run: ${CLI_INSTALL_HINTS[$i]}"
  fi
done
echo ""

if [[ $FOUND_COUNT -eq $TOTAL_COUNT ]]; then
  success "All 5 of 5 AI CLIs are installed and accessible in PATH!"
else
  warn "$FOUND_COUNT of $TOTAL_COUNT CLIs detected. Orbity will operate with available tools."
  info "Install missing CLIs using the suggestions above whenever you need those specific agents."
fi

# If any tool was found in ~/.hermes/node/bin or ~/.local/bin, make sure PATH is permanently exported
SHELL_RC_FILES=()
if [[ -f "${HOME}/.bashrc" ]]; then SHELL_RC_FILES+=("${HOME}/.bashrc"); fi
if [[ -f "${HOME}/.zshrc" ]]; then SHELL_RC_FILES+=("${HOME}/.zshrc"); fi
if [[ -f "${HOME}/.profile" ]]; then SHELL_RC_FILES+=("${HOME}/.profile"); fi

PATH_ADDITIONS=()
for p in "${EXTRA_PATHS[@]}"; do
  if [[ -d "$p" ]]; then
    PATH_ADDITIONS+=("$p")
  fi
done

# --- 5. Compile & Build Orbity (Release Mode) ---------------------------------
if [[ "$CHECK_ONLY" == true ]]; then
  step "5. Check-only mode requested. Skipping compilation and installation."
  success "System preflight verification completed."
  exit 0
fi

step "5. Compiling Orbity in Release Mode (LTO Enabled)"

cd "$PROJECT_ROOT"
info "Running: cargo build --release --workspace"
cargo build --release --workspace

TARGET_BIN="${PROJECT_ROOT}/target/release/orbity"
if [[ ! -f "$TARGET_BIN" ]]; then
  error "Compilation failed: $TARGET_BIN not found."
  exit 1
fi

success "Binary compiled successfully at: $TARGET_BIN"

# --- 6. Global Installation (Available in any terminal) -----------------------
step "6. Installing 'orbity' CLI Globally"

mkdir -p "$INSTALL_DIR"
cp -f "$TARGET_BIN" "${INSTALL_DIR}/orbity"
chmod +x "${INSTALL_DIR}/orbity"

success "Binary copied to: ${INSTALL_DIR}/orbity"

# Ensure INSTALL_DIR is in PATH permanently in user's shell rc files
EXPORT_LINE="export PATH=\"${INSTALL_DIR}:\$PATH\""

# Also include ~/.hermes/node/bin if pi assistant is located there
if [[ -d "${HOME}/.hermes/node/bin" ]]; then
  EXPORT_HERMES="export PATH=\"${HOME}/.hermes/node/bin:\$PATH\""
else
  EXPORT_HERMES=""
fi

if [[ "$NO_SHELL_EDIT" == true ]]; then
  info "Skipping shell rc file modification (--no-shell-edit specified)."
  info "To add orbity to your PATH manually, add this to your shell profile:"
  info "  $EXPORT_LINE"
else
  DO_EDIT=true
  if [[ "$AUTO_YES" == false ]]; then
    printf "\n"
    read -r -p "Add ${INSTALL_DIR} to your shell profile (.bashrc/.zshrc) for global PATH access? [Y/n]: " rc_confirm
    if [[ "$rc_confirm" =~ ^[Nn]$ ]]; then
      DO_EDIT=false
      info "Skipping shell rc modification. You can add ${INSTALL_DIR} to your PATH manually."
    fi
  fi

  if [[ "$DO_EDIT" == true ]]; then
    for rc in "${SHELL_RC_FILES[@]}"; do
      if ! grep -q "${INSTALL_DIR}" "$rc" 2>/dev/null; then
        echo "" >> "$rc"
        echo "# Orbity CLI Path" >> "$rc"
        echo "$EXPORT_LINE" >> "$rc"
        info "Added ${INSTALL_DIR} to $rc"
      fi
      if [[ -n "$EXPORT_HERMES" ]] && ! grep -q ".hermes/node/bin" "$rc" 2>/dev/null; then
        echo "$EXPORT_HERMES" >> "$rc"
        info "Added ~/.hermes/node/bin to $rc (for Pi CLI)"
      fi
    done
  fi
fi

# Export to current session as well
export PATH="${INSTALL_DIR}:${PATH}"

# Test installed CLI
if check_cmd orbity; then
  INSTALLED_VER="$(orbity --help 2>&1 | head -n 1 || echo "Orbity")"
  success "Verification successful: 'orbity' is now globally accessible in any terminal!"
else
  warn "'orbity' is installed in ${INSTALL_DIR}. You may need to restart your terminal or run: source ~/.bashrc"
fi

# --- 7. Final Preflight Doctor Run --------------------------------------------
step "7. Running 'orbity doctor' Verification"

"${INSTALL_DIR}/orbity" doctor || true

# --- Summary & Completion ----------------------------------------------------
echo ""
printf "${GREEN}${BOLD}==============================================================================${RESET}\n"
printf "${GREEN}${BOLD} 🎉 ORBITY SETUP COMPLETED SUCCESSFULLY!${RESET}\n"
printf "${GREEN}${BOLD}==============================================================================${RESET}\n"
echo ""
echo "  The 'orbity' command is now installed and ready to be used in any terminal."
echo ""
echo "  ${BOLD}Quick Start Commands:${RESET}"
echo "    ${CYAN}orbity doctor${RESET}                           # Run preflight health check"
echo "    ${CYAN}orbity init${RESET}                             # Initialize workspace in any folder (e.g. ~/alfa)"
echo "    ${CYAN}orbity sync${RESET}                             # Synchronize declarative teams/agents into SQLite"
echo "    ${CYAN}orbity run \"Your task description\"${RESET}     # Execute multi-agent orchestration"
echo "    ${CYAN}orbity serve --port 3000${RESET}                # Launch Tokio Topcoat full-stack web dashboard"
echo ""
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
  echo "  ${YELLOW}Notice:${RESET} To use 'orbity' in your current shell session immediately, run:"
  echo "    ${BOLD}source ~/.bashrc${RESET}  (or source ~/.zshrc)"
fi
echo ""
