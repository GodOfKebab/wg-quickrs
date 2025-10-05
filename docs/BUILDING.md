# Building wg-quickrs

_⚠️ This project is **under active development**.
Expect breaking changes and incomplete features._

This document is optimized for a fresh installation of Debian 13 that you would get when you spin up a VPS.

---

## 1. Installation

You can either build from scratch or use Docker.

Clone the repository:

```sh
sudo apt update
sudo apt install -y git
git clone https://github.com/GodOfKebab/wg-quickrs.git
cd wg-quickrs/src
```

You can either follow this document directly or use the following command to extract out the commands from this Markdown
document and run using the following script.
There are hidden comments on certain code snippets that allow the following command to extract out the important ones.
I use this to quickly initialize a server in the cloud.

```sh
# Local build
$SHELL run-md.sh ../docs/BUILDING.md install-deps-debian
. "$HOME/.cargo/env"
$SHELL run-md.sh ../docs/BUILDING.md build-src-debian
. ~/.bashrc
$SHELL run-md.sh ../docs/BUILDING.md run-agent-debian
$SHELL run-md.sh ../docs/BUILDING.md set-up-systemd-debian

# Docker
$SHELL run-md.sh ../docs/BUILDING.md build-src-docker
$SHELL run-md.sh ../docs/BUILDING.md run-agent-docker
```

---

### 1.1 Build from Scratch

#### Requirements

* Rust and Cargo
* Node.js/npm (for the web frontend)
* `iptables` (for setting NAT/port input allows for the agent)

The project is split into three parts:

* **`wg-quickrs-wasm`** – Rust code for the web frontend
* **`web`** – frontend assets
* **`wg-quickrs`** – backend, frontend server, and scripting tools bundled in `wg-quickrs` binary

---

#### 1.1.1 Install Rust/Cargo

[//]: # (install-deps-debian: 1.1.1 Install Rust/Cargo)

```sh
curl https://sh.rustup.rs -sSf | sh -s -- -y
. "$HOME/.cargo/env"
```

---

#### 1.1.2 Build `wg-quickrs-wasm`

Install `wasm-pack` dependency.

[//]: # (install-deps-debian: 1.1.2 Build wg-quickrs-wasm - Install 'wasm-pack' dependency)

```sh
cargo install wasm-pack
```

Build `wg-quickrs-wasm` directory.

[//]: # (build-src-debian: 1.1.2 Build wg-quickrs-wasm - Build 'wg-quickrs-wasm' directory.)

```sh
cd wg-quickrs-wasm
wasm-pack build --target wg-quickrs-web --out-dir ../wg-quickrs-web/pkg -- --features wasm --color=always
cd ..
```

---

#### 1.1.3 Build the web frontend

Install `npm` dependency.

[//]: # (install-deps-debian: 1.1.3 Build the web frontend - Install 'npm' dependency.)

```sh
sudo apt install -y npm
```

Build `web` directory.

[//]: # (build-src-debian: 1.1.3 Build the web frontend - Build 'web' directory.)

```sh
cd wg-quickrs-web
npm install
npm run build
cd ..
```

---

#### 1.1.4 Build and Install `wg-quickrs`

Install packages for the `aws-lc-sys` dependency.

[//]: # (install-deps-debian: 1.1.4 Build and Install wg-quickrs - Install packages for the 'aws-lc-sys' dependency.)

```sh
sudo apt-get update && sudo apt-get install -y musl-dev cmake clang llvm-dev libclang-dev pkg-config
```

Build the `wg-quickrs` directory.

This might take some time on slower machines.
The build process described here is simpler and slightly different from the ones in `.github/workflows/release.yml` and
`src/Dockerfile`. This is because those are optimized for cross-architecture/platform builds.
If the following method doesn't meet your needs, you can look into building with `Zig` as in those methods.

[//]: # (build-src-debian: 1.1.4 Build and Install wg-quickrs - build)

```sh
cargo build --release --package wg-quickrs --bin wg-quickrs

mkdir -p /usr/local/bin/
sudo install -m 755 target/release/wg-quickrs /usr/local/bin/
if ! printf %s "$PATH" | grep -q "/usr/local/bin"; then echo 'export PATH="/usr/local/bin:$PATH"' >> "$HOME/.profile"; fi
. $HOME/.profile
```

Install Bash/ZSH auto completions.

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

Check to make sure the script is accessible.

[//]: # (build-src-debian: 1.1.4 Build and Install wg-quickrs - sanity check)

```sh
wg-quickrs --help
# $ wg-quickrs
# A tool to manage the peer and network configuration of the WireGuard-based overlay network over the web console
# 
# Usage: wg-quickrs [OPTIONS] <COMMAND>
# 
# Commands:
#   init   Initialize the wg-quickrs agent.
#          Configuration options can be filled either by prompts on screen (when no argument is provided) or specified as arguments to this command
#   agent  Configure and run the wg-quickrs agent
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

#### 1.1.5 Cross-compilation

This portion uses `zigbuild` because the default rust toolchain was having trouble cross-compiling the `aws-lc-rs` dependency.

Install `zig` and `zigbuild`.

```sh
# ARCH options: x86_64, aarch64, arm based on your CURRENT machine you use to build binaries
# See all options at https://ziglang.org/download/
curl -L https://ziglang.org/download/0.15.1/zig-{{ ARCH }}-linux-0.15.1.tar.xz | tar -xJ
mv zig-* /usr/local/zig
ln -s /usr/local/zig/zig /usr/local/bin/zig
cargo install cargo-zigbuild
```

Build the `wg-quickrs` directory given a target platform.
Binary will be generated at `target/{{ TARGET }}/release/wg-quickrs`

```sh
# TARGET options: x86_64-unknown-linux-musl, aarch64-unknown-linux-musl, armv7-unknown-linux-musleabihf
# See all options by running the following
# rustup target list
rustup target add {{ TARGET }}
cargo zigbuild --release --package wg-quickrs --bin wg-quickrs --target={{ TARGET }}
```

---

#### 1.1.6 Configure TLS/HTTPS Certificates

I use the [tls-cert-generator](https://github.com/GodOfKebab/tls-cert-generator) to create TLS certificates locally.
See the documentation to generate certificates for other domains/servers.
Following grabs all the hostnames, IPv4+IPv6 interface addresses of the system and generates certificates for them.

[//]: # (install-deps-debian: 1.1.6 Configure TLS/HTTPS Certificates)

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

#### 1.1.7 Install WireGuard

Install packages for the `wg` and `wg-quick` dependency.

[//]: # (install-deps-debian: 1.1.7 Install WireGuard)

```sh
sudo apt install -y wireguard wireguard-tools
```

---

#### 1.1.8 Initialize and Configure the Agent

Run the following and follow the prompts to configure network, agent, and default peer settings when generating new
peers/connections.
This generates `/etc/wg-quickrs/conf.yml`, where all the settings/configurations are stored.
If you want to later edit the configuration, you can either use the scripting commands at `wg-quickrs agent <TAB>` or
manually edit this file and restart your agent.

[//]: # (run-agent-debian: 1.1.8 Initialize and Configure the Agent)

```sh
# Install to System:
wg-quickrs --wg-quickrs-config-folder /etc/wg-quickrs init
# Install to User:
# wg-quickrs init
```

---

#### 1.1.9 Run the Agent

Run the agent.

[//]: # (run-agent-debian: 1.1.9 Run the Agent)

```sh
# Run on System:
wg-quickrs --wg-quickrs-config-folder /etc/wg-quickrs agent run
# Run on User:
# wg-quickrs agent run
```

---

#### 1.1.10 Setup systemd service (optional)

Configure `systemd` for easily managing the agent.

Following creates:

* A user `wg-quickrs-user` with relatively weak privileges but a part of `wg-quickrs-group`
* A group `wg-quickrs-group` with
  * passwordless `sudo` access to `wg` and `wg-quick` executables
  * read/write/execute permissions for files under `/etc/wg-quickrs` and `/etc/wireguard`
* The systemd service `wg-quickrs` that is enabled and started
  * This service also gives necessary networking-related permissions.

[//]: # (set-up-systemd-debian: 1.1.10 Setup systemd service)

```sh
# setup a new user with weak permissions
sudo useradd --system --no-create-home --shell /usr/sbin/nologin --no-user-group wg-quickrs-user
sudo groupadd wg-quickrs-group
sudo usermod -aG wg-quickrs-group wg-quickrs-user
echo "wg-quickrs-user ALL=(ALL) NOPASSWD: $(which wg), $(which wg-quick)" | sudo tee /etc/sudoers.d/wg-quickrs
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
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_RAW CAP_NET_BIND_SERVICE

ExecStart=/usr/local/bin/wg-quickrs --wg-quickrs-config-folder /etc/wg-quickrs agent run
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

This might take some time on slower machines.
The build process of the Dockerfile is slightly different from the local build alternative because the Dockerfile is
optimized to build images for other architectures.

---

[Install Docker on Debian 13](https://docs.docker.com/engine/install/debian/#install-using-the-repository):

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

In the `docker-compose.init.yml` file:

* For the `tls-cert-generator` service (see [repo](https://github.com/GodOfKebab/tls-cert-generator)), edit the TLS
  certificate settings and enter the FQDN/Domain names for the certificates
* For the `wg-quickrs-init` service, edit the wg-quickrs settings

[//]: # (build-src-docker: 1.2 - Edit docker compose file)

```sh
cd ..
nano docker-compose.init.yml
```

Run the services to initialize the agent.

[//]: # (run-agent-docker: 1.2 - Generate certs)

```sh
docker compose -f docker-compose.init.yml up tls-cert-generator
docker compose -f docker-compose.init.yml up wg-quickrs-init

```

After initialization, you can run the `wg-quickrs-agent-run` service in `docker-compose.agent.yml`.

[//]: # (run-agent-docker: 1.2 - Run agent)

```sh
docker compose -f docker-compose.agent.yml up wg-quickrs-agent-run
```

