#!/usr/bin/env sh

# --- Detect rust target triple ---
if command -v rustc >/dev/null 2>&1; then
    target=$(rustc -vV | awk '/host:/ {print $2}')
else
    # fallback if rustc is not installed
    arch=$(uname -m)
    os=$(uname -s)
    case "$arch" in
        x86_64) cpu="x86_64" ;;
        aarch64|arm64) cpu="aarch64" ;;
        *) cpu="$arch" ;;
    esac
    case "$os" in
        Linux)   os_triple="unknown-linux-gnu" ;;
        Darwin)  os_triple="apple-darwin" ;;
        MINGW*|MSYS*|CYGWIN*) os_triple="pc-windows-msvc" ;;
        *) os_triple="unknown-$os" ;;
    esac
    target="${cpu}-${os_triple}"
fi

echo "Detected target: $target"

echo "Fetching latest release version..."
JSON=$(wget -qO- https://api.github.com/repos/GodOfKebab/wg-quickrs/releases/latest)
TAG=$(printf '%s\n' "$JSON" | grep '"tag_name":' | head -n1 | cut -d '"' -f4)
ASSET_URL=$(printf '%s\n' "$JSON" \
  | grep "browser_download_url" \
  | grep "aarch64-apple-darwin" \
  | cut -d '"' -f4)
echo "    Using latest release: $TAG"

INSTALL_DIR="$HOME/.wg-quickrs"
echo "Setting up and downloading the install directory at $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
wget -qO- "$ASSET_URL" | tar -xzf - -C "$INSTALL_DIR"
echo "    Done."

echo "Setting up TLS certs/keys at $INSTALL_DIR/certs..."

export COUNTRY="XX"
export STATE="XX"
export LOCALITY="XX"
export ORGANIZATION="XX"
export ORGANIZATIONAL_UNIT="XX"
export ROOT_CN="certificate-manager@XX"

(cd "$INSTALL_DIR" || exit ; wget -qO- https://raw.githubusercontent.com/GodOfKebab/certificate-manager/refs/heads/main/make-tls-certs.sh | sh -s all)
echo "    ✅ Generated TLS certs/keys"


echo "Setting up PATH and completions..."

current_shell=$(basename "$SHELL")

case "$current_shell" in
  bash)
    RC="$HOME/.bashrc"
    COMPLETION="$INSTALL_DIR/completions/wg-quickrs.bash"
    ;;
  zsh)
    RC="$HOME/.zshrc"
    COMPLETION="$INSTALL_DIR/completions/_wg-quickrs"
    ;;
  fish)
    RC="$HOME/.config/fish/config.fish"
    COMPLETION="$INSTALL_DIR/completions/wg-quickrs.fish"
    ;;
  elvish)
    RC="$HOME/.elvish/rc.elv"
    COMPLETION="$INSTALL_DIR/completions/wg-quickrs.elv"
    ;;
  *)
    RC=""
    ;;
esac

# Append PATH line if not already present
if [ -n "$RC" ]; then
  PATH_LINE="export PATH=\"$INSTALL_DIR/bin:\$PATH\""
  if ! grep -qxF "$PATH_LINE" "$RC" 2>/dev/null; then
    echo "### below is automatically added by wg-quickrs installer script ###" >> "$RC"
    echo "$PATH_LINE" >> "$RC"
  fi

  # Append completion line if not already present
  COMPLETION_LINE="source \"$COMPLETION\""
  if ! grep -qxF "$COMPLETION_LINE" "$RC" 2>/dev/null; then
        echo "### below is automatically added by wg-quickrs installer script ###" >> "$RC"
    echo "$COMPLETION_LINE" >> "$RC"
  fi

  echo "    ✅ Added PATH and completions to $RC"

  printf "\nOpen a new shell or run the following to use wg-quickrs command on this shell:\n\n"
  echo "    $PATH_LINE"
  echo "    $COMPLETION_LINE"
fi

printf "\nThen, you are ready to initialize your service with:\n\n"
printf "    wg-quickrs init\n"
printf "\nAfter a successful initialization, you can start up your service with:\n\n"
printf "    wg-quickrs agent run\n\n"

