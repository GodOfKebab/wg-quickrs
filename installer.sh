#!/usr/bin/env sh

usage() {
  cat << EOF
Installer script for wg-quickrs

Usage: $0 [COMMAND] [OPTIONS]

Commands:
  list-releases               List available releases

Options:
  -r, --release               Specify release
  -d, --dist-tarball PATH     Use local tarball instead of downloading from GitHub
  -i, --install-to LOCATION   Install location: system or user (default: system)
  --skip-deps                 Skip dependency installation (still checks if they exist)
  -h, --help                  Print help
EOF
  exit 1
}

ARG_RELEASE=""
ARG_DIST_TARBALL=""
ARG_INSTALL_TO="system"
ARG_SKIP_DEPS=0
ARG_COMMAND=""

# --- parse command (if first argument doesn't start with -) ---
if [ $# -gt 0 ] && [ "${1#-}" = "$1" ]; then
  ARG_COMMAND="$1"
  shift
fi

# --- parse options ---
while [ $# -gt 0 ]; do
  case "$1" in
    -r|--release) ARG_RELEASE="$2"; shift 2 ;;
    -d|--dist-tarball) ARG_DIST_TARBALL="$2"; shift 2 ;;
    -i|--install-to) ARG_INSTALL_TO="$2"; shift 2 ;;
    --skip-deps) ARG_SKIP_DEPS=1; shift ;;
    -h|--help) usage ;;
    --) shift; break ;;
    -*) echo "Unknown option: $1" >&2; usage ;;
    *) break ;;
  esac
done

list_releases() {
  echo "⏳ Fetching available releases..."
  RELEASES=$(wget -qO- "https://api.github.com/repos/GodOfKebab/wg-quickrs/releases?per_page=10" | grep '"tag_name"' | sed 's/.*"tag_name": "\([^"]*\)".*/\1/')
  echo "ℹ️ Available releases:"
  for tag in $RELEASES; do
      echo "    - $tag"
  done
}

# --- handle commands ---
if [ -n "$ARG_COMMAND" ]; then
  case "$ARG_COMMAND" in
    list-releases)
      list_releases
      exit 0
      ;;
    *)
      echo "❌ Unknown command: $ARG_COMMAND"
      usage
      ;;
  esac
fi

echo "✨  Welcome to wg-quickrs installer!"

# --- validate install-to argument ---
if [ "$ARG_INSTALL_TO" != "system" ] && [ "$ARG_INSTALL_TO" != "user" ]; then
  echo "❌ Invalid --install-to value: $ARG_INSTALL_TO (must be 'system' or 'user')"
  exit 1
fi

TARBALL_PATH=""
CLEANUP_TARBALL=0

tarball_cleanup() {
  if [ $CLEANUP_TARBALL -eq 1 ]; then
    echo "⏳ Cleaning up downloaded tarball..."
    rm -f "$TARBALL_PATH"
    echo "    ✅ Cleaned up tarball"
  fi
}

# --- privilege escalation helper ---
PRIVILEGE_CMD=""
run_privileged() {
  if [ -z "$PRIVILEGE_CMD" ]; then
    if command -v sudo >/dev/null 2>&1; then
      PRIVILEGE_CMD="sudo"
    elif command -v doas >/dev/null 2>&1; then
      PRIVILEGE_CMD="doas"
    else
      echo "⚠️ Neither sudo nor doas found. Cannot elevate privileges."
    fi
  fi
  "$PRIVILEGE_CMD" "$@"
}

# --- dependency check functions ---
WG_QUICKRS_PRIVILEGE_CMD=""
check_command() {
  # Try command -v first (may not exist)
  command -v "$1" >/dev/null 2>&1 && return 0

  if run_privileged sh -c "command -v '$1'" >/dev/null 2>&1; then
    echo "    ⚠️  command $1 is only reachable with $PRIVILEGE_CMD. You may need to run wg-quickrs with $PRIVILEGE_CMD."
    WG_QUICKRS_PRIVILEGE_CMD="$PRIVILEGE_CMD "
    return 0
  fi

  # Fallback to which (usually available in BusyBox)
  which "$1" >/dev/null 2>&1 && return 0
}

install_with_brew() {
  package="$1"
  echo "    ⏳ Installing these packages with Homebrew: $package..."
  if ! brew install "$package"; then
    echo "    ❌ Failed to install $package"
    echo "    ℹ️  You can use --skip-deps to skip dependency installation and install manually"
    return 1
  fi
  echo "    ✅ Installed: $package"
  return 0
}

install_with_apt() {
  package="$1"
  echo "    ⏳ Installing these packages with apt-get: $package..."

  # Try without privileges first
  if apt-get update -qq && apt-get install "$package"; then
    echo "    ✅ Installed: $package"
    return 0
  fi

  # If that failed, try with elevated privileges
  echo "    🔐 Administrator privileges my be required with apt"
  if run_privileged apt-get update -qq && run_privileged apt-get install "$package"; then
    echo "    ✅ Installed: $package"
    return 0
  fi

  echo "    ❌ Failed to install: $package"
  echo "    ℹ️  You can use --skip-deps to skip dependency installation and install manually"
  echo "Exiting."
  exit 1
}

install_with_apk() {
  package="$1"
  echo "    ⏳ Installing these packages with apk: $package..."

  # Try without privileges first
  if apk add -U --no-cache "$package"; then
    echo "    ✅ Installed: $package"
    return 0
  fi

  # If that failed, try with elevated privileges
  echo "    🔐 Administrator privileges my be required with apk"
  if run_privileged apk add -U --no-cache "$package"; then
    echo "    ✅ Installed: $package"
    return 0
  fi

  echo "    ❌ Failed to install: $package"
  echo "    ℹ️  You can use --skip-deps to skip dependency installation and install manually"
  echo "Exiting."
  exit 1
}

check_dependencies() {
  echo "⏳ Checking system dependencies..."

  if [ $ARG_SKIP_DEPS -eq 1 ]; then
    echo "    ℹ️  Dependency installation will be skipped (--skip-deps)"
  fi

  # Detect OS
  os=$(uname -s)

  case "$os" in
    Darwin)
      # macOS: check for wg (WireGuard)
      if ! check_command wg; then
        echo "    ⚠️  WireGuard (wg) not found"
        if [ $ARG_SKIP_DEPS -eq 1 ]; then
          echo "    ⏭️  Skipping installation of wireguard-tools (--skip-deps)"
        else
          if ! check_command brew; then
            echo "    ❌ Homebrew is required to install WireGuard but is not installed"
            echo "    ℹ️  Install Homebrew from https://brew.sh/"
            echo "    ℹ️  Alternatively, use --skip-deps to skip dependency installation and install WireGuard manually"
            exit 1
          fi
          install_with_brew wireguard-tools || exit 1
          check_command wg
        fi
      else
        echo "    ✅ WireGuard (wg) found"
      fi
      ;;

    Linux)
      # Linux: check for wg, resolvconf, ip, iptables
      missing_deps=""

      if ! check_command ip; then
        echo "    ⚠️  ip not found"
        missing_deps="$missing_deps iproute2"
      else
        echo "    ✅ ip found"
      fi

      if ! check_command resolvconf; then
        echo "    ⚠️  resolvconf not found"
        missing_deps="$missing_deps openresolv"
      else
        echo "    ✅ resolvconf found"
      fi

      if ! check_command iptables; then
        echo "    ⚠️  iptables not found"
        missing_deps="$missing_deps iptables"
      else
        echo "    ✅ iptables found"
      fi

      if ! check_command wg; then
        echo "    ⚠️  wg not found"
        if check_command apt-get; then
          missing_deps="$missing_deps wireguard"
        elif check_command apk; then
          missing_deps="$missing_deps wireguard-tools-wg"
        fi
      else
        echo "    ✅ wg found"
      fi

      # Install missing dependencies
      if [ -n "$missing_deps" ]; then
        if [ $ARG_SKIP_DEPS -eq 1 ]; then
          echo "    ⏭️  Skipping installation of:$missing_deps (--skip-deps)"
        else
          # Detect which package manager to use
          if check_command apt-get; then
            for dep in $missing_deps; do
                install_with_apt "$dep"
                check_command "$dep"
            done
          elif check_command apk; then
            for dep in $missing_deps; do
                install_with_apk "$dep"
                check_command "$dep"
            done
          else
            echo "    ❌ Neither apt-get nor apk package manager found"
            echo "    ℹ️  You can use --skip-deps to skip dependency installation and install manually"
            echo "Exiting."
            exit 1
          fi
        fi
      fi
      ;;

    *)
      echo "    ❌ Unknown OS: $os. This script is only supports Linux and macOS."
      echo "Exiting."
      exit 1
      ;;
  esac

  echo "✅ All dependencies satisfied"
}

# --- check dependencies ---
check_dependencies

# --- validate local tarball if provided ---
if [ -n "$ARG_DIST_TARBALL" ]; then
  if [ ! -f "$ARG_DIST_TARBALL" ]; then
    echo "❌ Tarball file not found: $ARG_DIST_TARBALL"
    exit 1
  fi
  echo "✅ Using local tarball: $ARG_DIST_TARBALL"
fi

# --- get ARG_RELEASE ---
if [ -z "$ARG_DIST_TARBALL" ]; then
  if [ -z "$ARG_RELEASE" ]; then
    echo "ℹ️ No release version specified. If you want to use a different version, specify like the following"
    echo
    echo "    installer.sh --release v1.0.0"
    echo
    list_releases
    echo "⏳ Fetching latest release version..."
    JSON=$(wget -qO- https://api.github.com/repos/GodOfKebab/wg-quickrs/releases/latest)
    ARG_RELEASE=$(printf '%s\n' "$JSON" | grep '"tag_name":' | head -n1 | cut -d '"' -f4)
    echo "    ✅ Using latest release: $ARG_RELEASE"
  else
    JSON=$(wget -qO- "https://api.github.com/repos/GodOfKebab/wg-quickrs/releases/tags/$ARG_RELEASE")
    if [ -z "$JSON" ]; then
      echo "    ❌ Failed to find the manually specified release: $ARG_RELEASE"
      echo
      list_releases
      exit 1;
    else
      echo "    ✅ Using manually specified release: $ARG_RELEASE"
    fi
  fi

  # --- detect rust target triple ---
  arch=$(uname -m)
  os=$(uname -s)
  os_triple="unknown-$os"
  case "$arch" in
      x86_64)
          cpu="x86_64"
          case "$os" in
              Linux)   os_triple="unknown-linux-musl" ;;
              Darwin)  os_triple="apple-darwin" ;;
              *) os_triple="unknown-$os" ;;
          esac
        ;;
      aarch64|arm64)
          cpu="aarch64"
          case "$os" in
              Linux)   os_triple="unknown-linux-musl" ;;
              Darwin)  os_triple="apple-darwin" ;;
              *) os_triple="unknown-$os" ;;
          esac
          ;;
      armv7l)
          cpu="armv7"
          case "$os" in
              Linux)   os_triple="unknown-linux-musleabihf" ;;
              Darwin)  os_triple="apple-darwin" ;;
              *) os_triple="unknown-$os" ;;
          esac
        ;;
      *) cpu="$arch" ;;
  esac
  target="${cpu}-${os_triple}"
  echo "✅ Detected target: $target"

  # --- find asset url ---
  echo "⏳ Fetching assets from the $ARG_RELEASE release..."

  ASSET_URL=$(printf '%s\n' "$JSON" \
    | grep "browser_download_url" \
    | grep "$target" \
    | cut -d '"' -f4)

  if [ -z "$ASSET_URL" ]; then
    echo "    ❌ Failed to find the correct asset from the $ARG_RELEASE release"
    ASSET_URL=$(printf '%s\n' "$JSON" \
      | grep "browser_download_url" \
      | cut -d '"' -f4)
    echo "    ℹ️ Here is a list of all available assets in the $ARG_RELEASE release:"
    for url in $ASSET_URL; do
        echo "        - $(echo "$url" | cut -d'/' -f9-)"
    done
    exit 1;
  else
    echo "    ✅ Detected asset $(echo "$ASSET_URL" | cut -d'/' -f9-) in the $ARG_RELEASE release"
  fi
fi

# --- download tarball if needed ---
download_tarball() {
    echo "⏳ Downloading release tarball to $TARBALL_PATH..."
    CLEANUP_TARBALL=1
    if ! wget -q -O "$TARBALL_PATH" "$ASSET_URL"; then
      echo "    ❌ Failed to download release from $ASSET_URL"
      tarball_cleanup
      exit 1
    fi
    echo "    ✅ Downloaded release tarball"
}


if [ -z "$ARG_DIST_TARBALL" ]; then
  # Download tarball from GitHub to current directory
  TARBALL_PATH="./wg-quickrs-$ARG_RELEASE.tar.gz"

  # Check if tarball already exists
  if [ -f "$TARBALL_PATH" ]; then
    printf "⚠️  Tarball %s already exists. Do you want to override it? [y/N]: " "$TARBALL_PATH"
    read override_tarball
    override_tarball=${override_tarball:-n}

    if [ "$override_tarball" = "y" ] || [ "$override_tarball" = "Y" ]; then
      download_tarball
    else
      echo "    ✅ Using existing tarball: $TARBALL_PATH"
    fi
  else
    download_tarball
  fi
else
  # Use provided local tarball
  TARBALL_PATH="$ARG_DIST_TARBALL"
fi

# --- set up WG_QUICKRS_INSTALL_DIR ---
WG_QUICKRS_REQUIRES_SUDO=0

if [ "$ARG_INSTALL_TO" = "system" ]; then
  WG_QUICKRS_INSTALL_DIR="/etc/wg-quickrs"
  WG_QUICKRS_INSTALL_DIR_OPTION=""
  echo "⏳ Installing configuration files to: $WG_QUICKRS_INSTALL_DIR (system)"
else
  WG_QUICKRS_INSTALL_DIR="$HOME/.wg-quickrs"
  WG_QUICKRS_INSTALL_DIR_OPTION=" --wg-quickrs-config-folder $WG_QUICKRS_INSTALL_DIR"
  echo "⏳ Installing configuration files to: $WG_QUICKRS_INSTALL_DIR (user)"
fi

install_from_tarball() {
  # Ensure target directory exists
  if ! mkdir -p "$WG_QUICKRS_INSTALL_DIR"; then
    echo "🔐 Administrator privileges may be required to create $WG_QUICKRS_INSTALL_DIR"
    if ! run_privileged mkdir -p "$WG_QUICKRS_INSTALL_DIR"; then
      echo "    ❌ Failed to create $WG_QUICKRS_INSTALL_DIR"
      tarball_cleanup
      exit 1
    fi
  fi

  # Extract tarball with elevated privileges if needed
  if [ ! -w "$WG_QUICKRS_INSTALL_DIR" ]; then
    echo "🔐 Administrator privileges required to extract to $WG_QUICKRS_INSTALL_DIR"
    if ! run_privileged tar -xzf "$TARBALL_PATH" -C "$WG_QUICKRS_INSTALL_DIR"; then
      echo "    ❌ Failed to extract tarball to $WG_QUICKRS_INSTALL_DIR"
      tarball_cleanup
      exit 1
    fi
    WG_QUICKRS_REQUIRES_SUDO=1
    WG_QUICKRS_PRIVILEGE_CMD="$PRIVILEGE_CMD "
    echo "    ⚠️  $WG_QUICKRS_INSTALL_DIR requires $PRIVILEGE_CMD. You may need to run wg-quickrs with $PRIVILEGE_CMD."
  else
    if ! tar -xzf "$TARBALL_PATH" -C "$WG_QUICKRS_INSTALL_DIR"; then
      echo "    ❌ Failed to extract tarball to $WG_QUICKRS_INSTALL_DIR"
      tarball_cleanup
      exit 1
    fi
  fi

  echo "    ✅ Extracted wg-quickrs from tarball to $WG_QUICKRS_INSTALL_DIR"
}

if [ "$ARG_INSTALL_TO" = "system" ]; then
  BIN_DIR="/usr/local/bin"
else
  BIN_DIR="$HOME/.local/bin"
fi

install_bin() {
  # Ensure target directory exists
  if ! mkdir -p "$BIN_DIR"; then
    echo "🔐 Administrator privileges may be required to create $BIN_DIR"
    if ! run_privileged mkdir -p "$BIN_DIR"; then
      echo "    ❌ Failed to create $BIN_DIR"
      tarball_cleanup
      exit 1
    fi
  fi

  # Install binary with elevated privileges if needed
  if [ ! -w "$BIN_DIR" ]; then
    echo "🔐 Administrator privileges required to install to $BIN_DIR"
    if ! run_privileged mv "$WG_QUICKRS_INSTALL_DIR/bin/wg-quickrs" "$BIN_DIR/wg-quickrs"; then
      echo "    ❌ Failed to install to $BIN_DIR - insufficient permissions"
      tarball_cleanup
      exit 1
    fi
  else
    if ! mv "$WG_QUICKRS_INSTALL_DIR/bin/wg-quickrs" "$BIN_DIR/wg-quickrs"; then
      echo "    ❌ Failed to install binary to $BIN_DIR"
      tarball_cleanup
      exit 1
    fi
  fi

  echo "    ✅ Installed wg-quickrs binary to $BIN_DIR"
  rm -rf "$WG_QUICKRS_INSTALL_DIR/bin"
}

if [ -n "$(ls -A "$WG_QUICKRS_INSTALL_DIR" 2>/dev/null)" ]; then
  printf "    ⚠️ Files already exist in %s. Do you want to override them? [y/N]: " "$WG_QUICKRS_INSTALL_DIR"
  read override
  override=${override:-n}
  if [ "$override" = "y" ] || [ "$override" = "Y" ]; then
    install_from_tarball
    install_bin
    echo "    ✅ Overwritten and updated files."
  else
    echo "Exiting..."
    tarball_cleanup
    exit
  fi
else
  install_from_tarball
  install_bin
  echo "    ✅ Fresh install completed."
fi

# Clean up downloaded tarball if we downloaded one
tarball_cleanup

echo "⏳ Setting up shell completions..."
case $(basename "$SHELL") in
  bash)
    BASH_COMPLETIONS_DIR="$HOME/.local/share/bash-completion/completions"
    mkdir -p "$BASH_COMPLETIONS_DIR"
    cp "$WG_QUICKRS_INSTALL_DIR/completions/wg-quickrs.bash" "$BASH_COMPLETIONS_DIR"
    echo "    ✅ Set bash shell completion at $BASH_COMPLETIONS_DIR/wg-quickrs.bash"
    echo "ℹ️ To use completion in this shell, you may need to run:"
    echo
    echo "    . ~/.bashrc"
    echo
    ;;
  zsh)
    ZSH_COMPLETIONS_DIR="$HOME/.zsh/completions"
    mkdir -p "$ZSH_COMPLETIONS_DIR"
    cp "$WG_QUICKRS_INSTALL_DIR/completions/_wg-quickrs" "$ZSH_COMPLETIONS_DIR"
    echo "    ✅ Set zsh shell completion at $ZSH_COMPLETIONS_DIR/_wg-quickrs"
    echo "ℹ️ To use completion in this shell, you may need to run:"
    echo
    echo "    . ~/.zshrc"
    echo
    ;;
  *)
    printf "    ⚠️ You are not using a supported shell (bash/zsh) for shell completions. Skipping shell completions.\n"
    ;;
esac

printf "🤔 Do you want to set up TLS certs/keys (at \"%s/certs\") now? [Y/n]: " "$WG_QUICKRS_INSTALL_DIR"
read setup_certs
setup_certs=${setup_certs:-y}

if [ "$setup_certs" = "y" ] || [ "$setup_certs" = "Y" ]; then
  if [ $WG_QUICKRS_REQUIRES_SUDO -eq 1  ]; then
    run_privileged mkdir -p "$WG_QUICKRS_INSTALL_DIR/certs"
    run_privileged wget -q https://github.com/GodOfKebab/tls-cert-generator/releases/download/v1.3.1/tls-cert-generator.sh -O "$WG_QUICKRS_INSTALL_DIR/certs/tls-cert-generator.sh"
  else
    mkdir -p "$WG_QUICKRS_INSTALL_DIR/certs"
    wget -q https://github.com/GodOfKebab/tls-cert-generator/releases/download/v1.3.1/tls-cert-generator.sh -O "$WG_QUICKRS_INSTALL_DIR/certs/tls-cert-generator.sh"
  fi

  echo "🔐 Enter server names for certificate generation:"
  echo "    - Specific hostnames/IPs (space-separated): example.com 192.168.1.1"
  echo "    - Special values (space-separated): all all-ipv4 all-ipv6 all-hostname"
  echo "    - Combined (space-separated): all example.com 192.168.1.1"
  printf "Servers (default: all) : "
  read server_names
  server_names=${server_names:-"all"}
  echo "⏳ Generating certificates for: $server_names"
  if [ $WG_QUICKRS_REQUIRES_SUDO -eq 1  ]; then
    run_privileged sh "$WG_QUICKRS_INSTALL_DIR/certs/tls-cert-generator.sh" -o "$WG_QUICKRS_INSTALL_DIR/certs" "$server_names"
  else
    sh "$WG_QUICKRS_INSTALL_DIR/certs/tls-cert-generator.sh" -o "$WG_QUICKRS_INSTALL_DIR/certs" "${server_names}"
  fi
  echo "✅ Generated TLS certs/keys"

  echo "ℹ️ If you want to generate cert/key in the future, run the following with YOUR_SERVER1, YOUR_SERVER2, etc. filled in"
  echo
  printf "    sh \"%s/certs/tls-cert-generator.sh\" -o \"%s/certs\" YOUR_SERVER1 YOUR_SERVER2\n" "$WG_QUICKRS_INSTALL_DIR" "$WG_QUICKRS_INSTALL_DIR"
  echo
else
  echo "⚠️ Skipping TLS cert setup. Remember to configure certs later!"
fi

# --- setup systemd service (optional) ---
if [ "$ARG_INSTALL_TO" = "system" ] && check_command systemctl; then
  printf "🔧 Do you want to set up systemd service for wg-quickrs? [Y/n]: "
  read setup_systemd
  setup_systemd=${setup_systemd:-y}

  if [ "$setup_systemd" = "y" ] || [ "$setup_systemd" = "Y" ]; then
    echo "⏳ Setting up systemd service..."

    # Create user and group
    if ! id wg-quickrs-user >/dev/null 2>&1; then
      echo "    Creating wg-quickrs-user..."
      if ! run_privileged useradd --system --no-create-home --shell /usr/sbin/nologin --no-user-group wg-quickrs-user; then
        echo "        ❌ Failed to create wg-quickrs-user"
        echo "        ⚠️  Skipping systemd setup"
      else
        echo "        ✅ Created wg-quickrs-user"
      fi
    else
      echo "        ✅ wg-quickrs-user already exists"
    fi

    if ! getent group wg-quickrs-group >/dev/null 2>&1; then
      echo "    Creating wg-quickrs-group..."
      if ! run_privileged groupadd wg-quickrs-group; then
        echo "        ❌ Failed to create wg-quickrs-group"
        echo "        ⚠️  Skipping systemd setup"
      else
        echo "        ✅ Created wg-quickrs-group"
      fi
    else
      echo "        ✅ wg-quickrs-group already exists"
    fi

    # Add user to group
    echo "    Adding wg-quickrs-user to wg-quickrs-group..."
    run_privileged usermod -aG wg-quickrs-group wg-quickrs-user

    # Setup sudoers
    echo "    Setting up passwordless sudo only for the wg-quickrs command..."
    echo "wg-quickrs-user ALL=(ALL) NOPASSWD: $BIN_DIR/wg-quickrs" | run_privileged tee /etc/sudoers.d/wg-quickrs >/dev/null
    run_privileged chmod 440 /etc/sudoers.d/wg-quickrs

    # Setup file permissions
    echo "    Setting up file permissions for $WG_QUICKRS_INSTALL_DIR..."
    run_privileged chown -R "$USER:wg-quickrs-group" "$WG_QUICKRS_INSTALL_DIR"
    run_privileged chmod -R 770 "$WG_QUICKRS_INSTALL_DIR"

    # Create systemd service file
    echo "    Creating systemd service file..."
    run_privileged tee /etc/systemd/system/wg-quickrs.service >/dev/null <<EOF
[Unit]
Description=wg-quickrs - An intuitive and feature-rich WireGuard configuration management tool
After=network.target

[Service]
Type=simple
User=wg-quickrs-user
Group=wg-quickrs-group
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_RAW CAP_NET_BIND_SERVICE

ExecStart=sudo $BIN_DIR/wg-quickrs agent run
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

    # Reload, enable, and start the service
    echo "    Reloading systemd service..."
    run_privileged systemctl daemon-reload

    echo "✅ systemd service setup complete"
    echo "ℹ️  After initializing your agent, you can manage the service with:"
    echo
    printf "        %ssystemctl enable wg-quickrs\n" "$WG_QUICKRS_PRIVILEGE_CMD"
    printf "        %ssystemctl start wg-quickrs\n" "$WG_QUICKRS_PRIVILEGE_CMD"
    printf "        %ssystemctl status wg-quickrs\n" "$WG_QUICKRS_PRIVILEGE_CMD"
    printf "        %ssystemctl stop wg-quickrs\n" "$WG_QUICKRS_PRIVILEGE_CMD"
    printf "        %ssystemctl restart wg-quickrs\n" "$WG_QUICKRS_PRIVILEGE_CMD"
    printf "        %sjournalctl -u wg-quickrs.service -n 50\n" "$WG_QUICKRS_PRIVILEGE_CMD"
    echo
  else
    echo "⚠️ Skipping systemd setup"
  fi
fi

# --- check if wg-quickrs is in PATH ---
if ! wg-quickrs --help >/dev/null 2>&1; then
  echo "⚠️  wg-quickrs is not in your PATH. Please add $BIN_DIR to your PATH:"
  echo
  echo "        export PATH=\"\$PATH:$BIN_DIR\""
  echo
fi

printf "🛠️ You are ready to initialize your agent with:\n\n"
printf "    %swg-quickrs%s agent init\n" "$WG_QUICKRS_PRIVILEGE_CMD" "$WG_QUICKRS_INSTALL_DIR_OPTION"
printf "\n🚀 After a successful initialization, you can start up your service with:\n\n"
printf "    %swg-quickrs%s agent run\n\n" "$WG_QUICKRS_PRIVILEGE_CMD" "$WG_QUICKRS_INSTALL_DIR_OPTION"

