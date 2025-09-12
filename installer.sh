#!/usr/bin/env sh

# --- Detect rust target triple ---
# fallback if rustc is not installed
arch=$(uname -m)
os=$(uname -s)
case "$arch" in
    x86_64) cpu="x86_64" ;;
    aarch64|arm64) cpu="aarch64" ;;
    *) cpu="$arch" ;;
esac
case "$os" in
    Linux)   os_triple="unknown-linux-musl" ;;
    Darwin)  os_triple="apple-darwin" ;;
    *) os_triple="unknown-$os" ;;
esac
target="${cpu}-${os_triple}"

echo "✅ Detected target: $target"

echo "⏳ Fetching latest release version..."
JSON=$(wget -qO- https://api.github.com/repos/GodOfKebab/wg-quickrs/releases/latest)
TAG=$(printf '%s\n' "$JSON" | grep '"tag_name":' | head -n1 | cut -d '"' -f4)
ASSET_URL=$(printf '%s\n' "$JSON" \
  | grep "browser_download_url" \
  | grep "$target" \
  | cut -d '"' -f4)
echo "    ✅ Using latest release: $TAG"

INSTALL_DIR="$HOME/.wg-quickrs"
echo "⏳ Setting up and downloading the install directory at $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
if [ -n "$(ls -A "$INSTALL_DIR" 2>/dev/null)" ]; then
  printf "⚠️ Files already exist in %s. Do you want to overwrite them? [y/N]: " "$INSTALL_DIR"
  read overwrite
  overwrite=${overwrite:-n}
  if [ "$overwrite" = "y" ] || [ "$overwrite" = "Y" ]; then
    wget -qO- "$ASSET_URL" | tar -xzf - -C "$INSTALL_DIR"
    echo "    ✅ Overwritten and updated files."
  else
    echo "Exiting..."
    exit
  fi
else
  wget -qO- "$ASSET_URL" | tar -xzf - -C "$INSTALL_DIR"
  echo "    ✅ Fresh install completed."
fi

printf "🤔 Do you want to set up TLS certs/keys now? [Y/n]: "
read setup_certs
setup_certs=${setup_certs:-y}

if [ "$setup_certs" = "y" ] || [ "$setup_certs" = "Y" ]; then
  echo "⏳ Setting up TLS certs/keys at $INSTALL_DIR/certs..."
  mkdir -p "$INSTALL_DIR/certs"
  wget -q https://raw.githubusercontent.com/GodOfKebab/tls-cert-generator/refs/heads/main/tls-cert-generator.sh -O "$INSTALL_DIR/certs/tls-cert-generator.sh"
  sh "$INSTALL_DIR/certs/tls-cert-generator.sh" -f -o "$INSTALL_DIR/certs" all
  echo "    ✅ Generated TLS certs/keys"
  echo "    ℹ️ If you want to generate cert/key for other servers, run the following with YOUR_SERVER1, YOUR_SERVER2, etc. filled in"
  echo
  echo "        sh $INSTALL_DIR/certs/tls-cert-generator.sh" -o "$INSTALL_DIR/certs" YOUR_SERVER1 YOUR_SERVER2
  echo
else
  echo "    ⚠️ Skipping TLS cert setup. Remember to configure certs later!"
fi


echo "⏳ Setting up PATH and completions..."

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

  echo "    📂 Added PATH and completions to $RC"

  printf "\n✨ Open a new shell or run the following to use wg-quickrs command on this shell:\n\n"
  echo "    $PATH_LINE"
  echo "    $COMPLETION_LINE"
fi

printf "\n🛠️ Then, you are ready to initialize your service with:\n\n"
printf "    wg-quickrs init\n"
printf "\n🚀 After a successful initialization, you can start up your service with:\n\n"
printf "    wg-quickrs agent run\n\n"

