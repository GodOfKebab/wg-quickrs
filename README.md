# wg-quickrs

[![License](https://img.shields.io/github/license/godofkebab/wg-quickrs?logo=GitHub&color=brightgreen)](https://github.com/GodOfKebab/wg-quickrs)
![Static Badge](https://img.shields.io/badge/amd64%20%7C%20arm64%20%7C%20arm%2Fv7%20%20-%20grey?label=arch)
![Static Badge](https://img.shields.io/badge/Linux%20%7C%20macOS%20%20-%20black?label=platform)

[![Release](https://img.shields.io/github/v/tag/godofkebab/wg-quickrs?logo=github&label=latest%20tag&color=blue)](https://github.com/godofkebab/wg-quickrs/releases/latest)
[![Docker](https://img.shields.io/docker/image-size/godofkebab/wg-quickrs?logo=docker&color=%232496ED)](https://hub.docker.com/repository/docker/godofkebab/wg-quickrs)
[![Docker](https://img.shields.io/docker/pulls/godofkebab/wg-quickrs?logo=docker&color=%232496ED)](https://hub.docker.com/repository/docker/godofkebab/wg-quickrs/tags)
![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FGodOfKebab%2Fwg-quickrs%2Frefs%2Fheads%2Fmain%2Fsrc%2Fwg-quickrs%2FCargo.toml&query=package.rust-version&logo=rust&label=rust&color=%23000000)
![Dynamic JSON Badge](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2FGodOfKebab%2Fwg-quickrs%2Frefs%2Fheads%2Fmain%2Fsrc%2Fwg-quickrs-web%2Fpackage.json&query=dependencies.vue&logo=vue.js&label=vue&color=%234FC08D)

✨ An intuitive multi-peer `wg` wrapper written in 🦀 Rust (`wg-quick` alternative).

⚡ Rust + Vue + WASM + WireGuard = 🧪 one static binary + 📝 one [YAML file](docs/SCHEMA.md) to rule them all 🪄

Run it on your [router](docs/quick-start/router.md), [server](docs/quick-start/server.md), or [docker host](docs/quick-start/docker.md) and manage your WireGuard VPN from a terminal or a web interface.

<p align="center">
  <img src="https://yasar.idikut.cc/project-assets/wg-quickrs-speedtest.gif" alt="speedtest demo">
</p>

<p align="center">
  <img src="https://yasar.idikut.cc/project-assets/wg-quickrs-demo.gif" alt="usage demo">
</p>

Features:
- Interactive graph to configure your P2P network
- HTTPS support and password login with JWT-based API authentication
- Automatic firewall/NAT setup (`iptables` for Debian/Linux or `pf` for macOS, both usually come preinstalled with the OS)
- If you are not feeling like dealing with VPN/networking on your machine, you can also just use the CLI or the web console to create `.conf` files/QR codes for your network peers.

---

To get started, see quick start guides for [routers](docs/quick-start/router.md), [servers](docs/quick-start/server.md), or [docker hosts](docs/quick-start/docker.md).
