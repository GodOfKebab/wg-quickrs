# Developing wg-quickrs

⚠️ This project is **under active development**.
Expect breaking changes and incomplete features.

This document is optimized for Debian 13.

---

## 1. Installation

You can either build from scratch or use Docker.

Clone the repository:

```sh
sudo apt install -y git
git clone https://github.com/GodOfKebab/wg-quickrs.git
cd wg-quickrs/src
```

---

### 1.1 Build from Scratch

#### Requirements

* `iptables`
* Rust and Cargo
* Node.js/npm (for the web frontend)

The project is split into:

* **`rust-wasm`** – Rust code for the web frontend
* **`web`** – frontend assets
* **`rust-agent`** – backend, frontend server, and scripting tools bundled in `wg-quickrs` binary

You can either follow this document directly or use the following command to extract out the commands using the
following command and run.
There are hidden comments on certain code snippets that allow the following command to extract out the important ones.
I use this to quickly initialize a server in the cloud.

```sh
awk -v tag="build-src-debian" '
  # Match lines like: [//]: # (build-src-debian: description)
  $0 ~ "^[[]//[]]: # \\(" tag ": " {
      desc = substr($0, length(tag)+12, length($0)-length(tag)-12)
      tagged=1
      next
  }

  /^```sh$/ && tagged {
      inblock=1
      tagged=0
      print "echo " "\"" desc "\""
      next
  }

  /^```/ && inblock {
      inblock=0
      print ""
      next
  }

  inblock { print }
' ../dev/DEVELOPING.md > .build-agent.sh
bash .build-agent.sh
```
---

#### 1.1.1 Install Rust/Cargo

[//]: # (build-src-debian: 1.1.1 Install Rust/Cargo)

```sh
curl https://sh.rustup.rs -sSf | sh -s -- -y
. "$HOME/.cargo/env"
```

---

#### 1.1.2 Build `rust-wasm`

[//]: # (build-src-debian: 1.1.2 Build rust-wasm)

```sh
cargo install wasm-pack
cd rust-wasm
wasm-pack build --target web --out-dir ../web/pkg -- --features wasm --color=always
cd ..
```

---

#### 1.1.3 Build the web frontend

[//]: # (build-src-debian: 1.1.3 Build the web frontend)

```sh
sudo apt install -y npm
cd web
npm install
npm run build
cd ..
```

---

#### 1.1.4 Build and Install `wg-quickrs`

This might take some time on slower machines.
The build process described here is simpler and slightly different from the ones in `.github/workflows/release.yml` and
`src/Dockerfile`.
This is because those are optimized for cross-platform builds.
If the following method doesn't meet your needs, you can look into building with `Zig` as in those methods.
If errors relating to `aws-lc-sys` are raised, make sure you have the required packages:

[//]: # (build-src-debian: 1.1.4 Build and Install wg-quickrs - install deps)

```sh
sudo apt-get update && sudo apt-get install -y musl-dev cmake clang llvm-dev libclang-dev pkg-config
```

[//]: # (build-src-debian: 1.1.4 Build and Install wg-quickrs - build)

```sh
cargo build --release --package wg-quickrs --bin wg-quickrs

mkdir -p /usr/local/bin/
sudo install -m 755 target/release/wg-quickrs /usr/local/bin/
if ! printf %s "$PATH" | grep -q "/usr/local/bin"; then echo 'export PATH="/usr/local/bin:$PATH"' >> "$HOME/.profile"; fi
. $HOME/.profile
```

[//]: # (build-src-debian: 1.1.4 Build and Install wg-quickrs - completions)

```sh
# Bash
cp target/release/completions/wg-quickrs.bash /etc/bash_completion.d/
. ~/.bashrc
```

```sh
# ZSH
mkdir -p ~/.zsh/completions
cp target/release/completions/_wg-quickrs ~/.zsh/completions/
grep -qxF 'fpath=(~/.zsh/completions $fpath)' ~/.zshrc || printf '\nfpath=(~/.zsh/completions $fpath)\nautoload -Uz compinit\ncompinit\n' >> ~/.zshrc
. ~/.zshrc
```

[//]: # (build-src-debian: 1.1.4 Build and Install wg-quickrs - sanity check)

```sh
wg-quickrs --help
# $ wg-quickrs
# A tool to manage the peer and network configuration of the WireGuard-based overlay network over the web console
# 
# Usage: wg-quickrs [OPTIONS] <COMMAND>
# 
# Commands:
#   init   Initialize the wg-quickrs rust-agent.
#          Configuration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command
#   agent  Configure and run the wg-quickrs rust-agent
#   help   Print this message or the help of the given subcommand(s)
# 
# Options:
#   -v, --verbose
#           Increase verbosity level from Info to Debug
#       --wg-quickrs-config-folder <WG_QUICKRS_CONFIG_FOLDER>
#           [default: .wg-quickrs]
#   -h, --help
#           Print help
#   -V, --version
#           Print version

# wg-quickrs <TAB>           # Shows available commands (init, agent)
# wg-quickrs agent <TAB>     # Shows available agent subcommands
# wg-quickrs init --<TAB>    # Shows available options for the init command
```

---

#### 1.1.5 Configure TLS/HTTPS Certificates

[//]: # (build-src-debian: 1.1.5 Configure TLS/HTTPS Certificates)

```sh
# Install to System:
sudo mkdir -p /etc/wg-quickrs/certs
wget https://raw.githubusercontent.com/GodOfKebab/tls-cert-generator/refs/heads/main/tls-cert-generator.sh -O /etc/wg-quickrs/certs/tls-cert-generator.sh
sh /etc/wg-quickrs/certs/tls-cert-generator.sh -o /etc/wg-quickrs/certs all

# Install to User:
# wget https://raw.githubusercontent.com/GodOfKebab/tls-cert-generator/refs/heads/main/tls-cert-generator.sh -O tls-cert-generator.sh
# mkdir -p $HOME/.wg-quickrs/certs && sh tls-cert-generator.sh -o $HOME/.wg-quickrs/certs all

```

---

#### 1.1.6 Install WireGuard

[//]: # (build-src-debian: 1.1.6 Install WireGuard)

```sh
sudo apt install -y wireguard wireguard-tools
```

---

#### 1.1.7 Initialize and Configure the Agent

[//]: # (build-src-debian: 1.1.7 Initialize and Configure the Agent)

```sh
# Install to System:
wg-quickrs --wg-quickrs-config-folder /etc/wg-quickrs init
# Install to User:
# wg-quickrs init
```

Follow the prompts to configure network, agent, and default peer settings. This generates `$HOME/.wg-quickrs/conf.yml`.

---

#### 1.1.8 Run the Agent

[//]: # (build-src-debian: 1.1.8 Run the Agent)

```sh
# Run on System:
wg-quickrs --wg-quickrs-config-folder /etc/wg-quickrs agent run
# Run on User:
# wg-quickrs agent run

```

* HTTP frontend/API: `http://<your-ip>:80/`
* HTTPS frontend/API: `https://<your-ip>:443/`
* WireGuard tunnel: `<public-ip>:51820`

---

#### 1.1.9 Setup systemd service (optional)

```sh
# setup a new user with less permissions
sudo useradd --system --no-create-home --shell /usr/sbin/nologin --no-user-group wg-quickrs-user
sudo groupadd wg-quickrs-group
sudo usermod -aG wg-quickrs-group wg-quickrs-user
echo "wg-quickrs-user ALL=(ALL) NOPASSWD: $(which wg), $(which wg-quick), $(which wg-quick)" | sudo tee /etc/sudoers.d/wg-quickrs
sudo chmod 440 /etc/sudoers.d/wg-quickrs

# setup file permissions
sudo chown -R $USER:wg-quickrs-group /etc/wg-quickrs
sudo chmod -R 770 /etc/wg-quickrs
sudo chown -R $USER:wg-quickrs-group /etc/wireguard
sudo chmod -R 770 /etc/wireguard

# setup systemd
sudo tee /etc/systemd/system/wg-quickrs.service > /dev/null <<'EOF'
[Unit]
Description=wg-quickrs - An intuitive and feature-rich WireGuard configuration management tool
After=network.target

[Service]
Type=simple
User=wg-quickrs-user
Group=wg-quickrs-group
ExecStart=/usr/local/bin/wg-quickrs --wg-quickrs-config-folder /etc/wg-quickrs agent run

AmbientCapabilities=CAP_NET_ADMIN CAP_NET_RAW CAP_NET_BIND_SERVICE

Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl daemon-reload
sudo systemctl enable wg-quickrs
sudo systemctl start wg-quickrs
sudo systemctl status wg-quickrs
# sudo journalctl -u wg-quickrs.service -n 50
```

---

### 1.2 Using Docker

You can either follow this document directly or use the following command to extract out the commands using the
following command and run.
There are hidden comments on certain code snippets that allow the following command to extract out the important ones.
I use this to quickly initialize a server in the cloud.

```sh
awk -v tag="build-src-docker" '
  # Match lines like: [//]: # (build-src-docker: description)
  $0 ~ "^[[]//[]]: # \\(" tag ": " {
      desc = substr($0, length(tag)+12, length($0)-length(tag)-12)
      tagged=1
      next
  }

  /^```sh$/ && tagged {
      inblock=1
      tagged=0
      print "echo " "\"" desc "\""
      next
  }

  /^```/ && inblock {
      inblock=0
      print ""
      next
  }

  inblock { print }
' ../dev/DEVELOPING.md > .build-docker.sh
bash .build-docker.sh
```

---

Install Docker on Debian 13:

[//]: # (build-src-docker: 1.2 - Install docker)

```sh
for pkg in docker.io docker-doc docker-compose podman-docker containerd runc; do sudo apt-get remove $pkg; done
sudo apt-get update
sudo apt-get install ca-certificates curl
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
sudo chmod a+r /etc/apt/keyrings/docker.asc

echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian $(. /etc/os-release && echo \"$VERSION_CODENAME\") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
```

Edit the TLS certificate settings and enter the FQDN/Domain names for the certificates from `tls-cert-generator`
service in `docker-compose.init.yml`.

[//]: # (build-src-docker: 1.2 - Edit docker compose file)

```sh
cd ..
nano docker-compose.init.yml
```

[//]: # (build-src-docker: 1.2 - Generate certs)

```sh
docker compose -f docker-compose.init.yml up tls-cert-generator
```

Edit the wg-quickrs settings from `wg-quickrs-init` service in `docker-compose.init.yml`.
Especially make sure that the IP addresses are updated and correct TLS cert/key paths are entered.

[//]: # (build-src-docker: 1.2 - Initialize agent)

```sh
docker compose -f docker-compose.init.yml up wg-quickrs-init
```

After initialization, you can run the `wg-quickrs-agent-run` service in `docker-compose.agent.yml`.

[//]: # (build-src-docker: 1.2 - Run agent)

```sh
docker compose -f docker-compose.agent.yml up wg-quickrs-agent-run
```

