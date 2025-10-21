#!/usr/bin/env sh

usage() {
  cat << EOF
Installer script for wg-quickrs

Usage: $0 [OPTIONS]

Options:
  -r, --release            Specify release
  -h, --help               Print help
EOF
  exit 1
}

ARG_RELEASE=""

# --- parse options ---
while [ $# -gt 0 ]; do
  case "$1" in
    -r|--release) ARG_RELEASE="$2"; shift 2 ;;
    -h|--help) usage ;;
    --) shift; break ;;
    -*) echo "Unknown option: $1" >&2; usage ;;
    *) break ;;
  esac
done

echo "✨  Welcome to wg-quickrs installer!"

# --- get ARG_RELEASE ---
if [ -z "$ARG_RELEASE" ]; then
  echo "ℹ️ No release version specified. If you want to use a different version, specify like the following"
  echo
  echo "    installer.sh --release v1.0.0"
  echo
  RELEASES=$(wget -qO- "https://api.github.com/repos/GodOfKebab/wg-quickrs/releases?per_page=10" | grep '"tag_name"' | sed 's/.*"tag_name": "\([^"]*\)".*/\1/')
  echo "ℹ️ Here is a list of available releases:"
  for tag in $RELEASES; do
      echo "    - $tag"
  done
  echo "⏳ Fetching latest release version..."
  JSON=$(wget -qO- https://api.github.com/repos/GodOfKebab/wg-quickrs/releases/latest)
  ARG_RELEASE=$(printf '%s\n' "$JSON" | grep '"tag_name":' | head -n1 | cut -d '"' -f4)
  echo "    ✅ Using latest release: $ARG_RELEASE"
else
  JSON=$(wget -qO- "https://api.github.com/repos/GodOfKebab/wg-quickrs/releases/tags/$ARG_RELEASE")
  if [ -z "$JSON" ]; then
    echo "    ❌ Failed to find the manually specified release: $ARG_RELEASE"
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

case "$os" in
  Linux)   WG_QUICKRS_INSTALL_DIR="/etc/wg-quickrs" ;;
  Darwin)  WG_QUICKRS_INSTALL_DIR="/opt/homebrew/etc/wg-quickrs" ;;
  *)       WG_QUICKRS_INSTALL_DIR="$HOME/.wg-quickrs" ;;
esac

# --- set up WG_QUICKRS_INSTALL_DIR ---
echo "⏳ Installing configuration files to: $WG_QUICKRS_INSTALL_DIR"
mkdir -p "$WG_QUICKRS_INSTALL_DIR"

SYSTEM_BIN_DIR="/usr/local/bin"
USER_BIN_DIR="$HOME/.local/bin"

install_bin() {
  if [ ! -w "$SYSTEM_BIN_DIR" ]; then
    echo "🔐 Administrator privileges (write permission to $SYSTEM_BIN_DIR) required to install to $SYSTEM_BIN_DIR"
    if ! sudo mv "$WG_QUICKRS_INSTALL_DIR/bin/wg-quickrs" "$SYSTEM_BIN_DIR/wg-quickrs" 2>/dev/null; then
      echo "    ⚠️  Failed to install to $SYSTEM_BIN_DIR, trying $USER_BIN_DIR instead"
      mkdir -p "$USER_BIN_DIR"
      if ! mv "$WG_QUICKRS_INSTALL_DIR/bin/wg-quickrs" "$USER_BIN_DIR/wg-quickrs"; then
        echo "        ❌ Failed to install binary"
        exit 1
      fi
      echo "        ✅ Installed wg-quickrs binary to $USER_BIN_DIR"

      # Warn about PATH if needed
      if ! echo "$PATH" | grep -q "$USER_BIN_DIR"; then
        echo "    ⚠️  Add $USER_BIN_DIR to your PATH:"
        echo
        echo "        export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo
      fi
    else
      echo "    ✅ Installed wg-quickrs binary to $SYSTEM_BIN_DIR"
    fi
    rm -r "$WG_QUICKRS_INSTALL_DIR/bin"
  else
    mv "$WG_QUICKRS_INSTALL_DIR/bin/wg-quickrs" "$SYSTEM_BIN_DIR/wg-quickrs"
    rm -r "$WG_QUICKRS_INSTALL_DIR/bin"
  fi
}

if [ -n "$(ls -A "$WG_QUICKRS_INSTALL_DIR" 2>/dev/null)" ]; then
  printf "    ⚠️ Files already exist in %s. Do you want to overwrite them? [y/N]: " "$WG_QUICKRS_INSTALL_DIR"
  read overwrite
  overwrite=${overwrite:-n}
  if [ "$overwrite" = "y" ] || [ "$overwrite" = "Y" ]; then
    wget -qO- "$ASSET_URL" | tar -xzf - -C "$WG_QUICKRS_INSTALL_DIR"
    install_bin
    echo "    ✅ Overwritten and updated files."
  else
    echo "Exiting..."
    exit
  fi
else
  wget -qO- "$ASSET_URL" | tar -xzf - -C "$WG_QUICKRS_INSTALL_DIR"
  install_bin
  echo "    ✅ Fresh install completed."
fi

printf "🤔 Do you want to set up TLS certs/keys now? [Y/n]: "
read setup_certs
setup_certs=${setup_certs:-y}

if [ "$setup_certs" = "y" ] || [ "$setup_certs" = "Y" ]; then
  mkdir -p "$WG_QUICKRS_INSTALL_DIR/certs"
  wget -q https://github.com/GodOfKebab/tls-cert-generator/releases/download/v1.3.1/tls-cert-generator.sh -O "$WG_QUICKRS_INSTALL_DIR/certs/tls-cert-generator.sh"

  echo "🔐 Enter server names for certificate generation:"
  echo "    - Specific hostnames/IPs (space-separated): example.com 192.168.1.1"
  echo "    - Special values (space-separated): all all-ipv4 all-ipv6 all-hostname"
  echo "    - Combined (space-separated): all example.com 192.168.1.1"
  printf "Servers (default: all) : "
  read server_names
  server_names=${server_names:-"all"}
  echo "⏳ Generating certificates for: $server_names"
  sh "$WG_QUICKRS_INSTALL_DIR/certs/tls-cert-generator.sh" -f -o "$WG_QUICKRS_INSTALL_DIR/certs" "$server_names"
  echo "✅ Generated TLS certs/keys"

  echo "ℹ️ If you want to generate cert/key in the future, run the following with YOUR_SERVER1, YOUR_SERVER2, etc. filled in"
  echo
  echo "    sh $WG_QUICKRS_INSTALL_DIR/certs/tls-cert-generator.sh" -o "$WG_QUICKRS_INSTALL_DIR/certs" YOUR_SERVER1 YOUR_SERVER2
  echo
else
  echo "⚠️ Skipping TLS cert setup. Remember to configure certs later!"
fi

echo "⏳ Setting up shell completions..."

current_shell=$(basename "$SHELL")

case "$current_shell" in
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
    echo "    ⚠️ You are not using a supported shell (bash/zsh) for shell completions. Skipping shell completions."
    ;;
esac


printf "\n🛠️ Then, you are ready to initialize your service with:\n\n"
printf "    wg-quickrs init\n"
printf "\n🚀 After a successful initialization, you can start up your service with:\n\n"
printf "    wg-quickrs agent run\n\n"

