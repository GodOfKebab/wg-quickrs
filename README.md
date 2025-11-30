# wg-quickrs

[![License](https://img.shields.io/github/license/godofkebab/wg-quickrs?logo=GitHub&color=brightgreen)](https://github.com/GodOfKebab/wg-quickrs)
![Static Badge](https://img.shields.io/badge/amd64%20%7C%20arm64%20%7C%20arm%2Fv7%20%20-%20grey?label=arch)
![Static Badge](https://img.shields.io/badge/Linux%20%7C%20macOS%20%20-%20black?label=platform)

[![Release](https://img.shields.io/github/v/tag/godofkebab/wg-quickrs?logo=github&label=latest%20tag&color=blue)](https://github.com/godofkebab/wg-quickrs/releases/latest)
[![Docker](https://img.shields.io/docker/image-size/godofkebab/wg-quickrs?logo=docker&color=%232496ED)](https://hub.docker.com/repository/docker/godofkebab/wg-quickrs)
[![Docker](https://img.shields.io/docker/pulls/godofkebab/wg-quickrs?logo=docker&color=%232496ED)](https://hub.docker.com/repository/docker/godofkebab/wg-quickrs/tags)
![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FGodOfKebab%2Fwg-quickrs%2Frefs%2Fheads%2Fmain%2Fsrc%2Fwg-quickrs%2FCargo.toml&query=package.rust-version&logo=rust&label=rust&color=%23000000)
![Dynamic JSON Badge](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2FGodOfKebab%2Fwg-quickrs%2Frefs%2Fheads%2Fmain%2Fsrc%2Fwg-quickrs-web%2Fpackage.json&query=dependencies.vue&logo=vue.js&label=vue&color=%234FC08D)
![Static Badge](https://img.shields.io/badge/%20wasm32-%23654ff0?logo=webassembly&labelColor=%23F0F0F0)

✨ An insanely simple and configurable [`wg`](https://www.wireguard.com)/[`awg`](https://docs.amnezia.org/documentation/amnezia-wg/) wrapper written in 🦀 Rust (`wg-quick` alternative).

⚡ Rust + WireGuard/[AmneziaWG](docs/notes/amneziawg.md) = 🧪 one [static binary](docs/notes/static-binary.md) + 📝 one [YAML file](docs/notes/schema.md) to rule them all 🪄

Be it deploying and managing a [road warrior](docs/notes/network-setup.md#road-warrior), [site-to-site](docs/notes/network-setup.md#site-to-site), or a [mesh](docs/notes/network-setup.md#mesh) network, `wg-quickrs` is here to help.

Run it on your [router](docs/quick-start/router.md), [server](docs/quick-start/server.md), or [docker host](docs/quick-start/docker.md) and manage WireGuard tunnels from a [terminal](docs/quick-start/cli.md) or a browser.

<p align="center">
  <img src="https://yasar.idikut.cc/project-assets/wg-quickrs-speedtest.gif" alt="speedtest demo">
</p>

<p align="center">
  <img src="https://yasar.idikut.cc/project-assets/wg-quickrs-demo.gif" alt="usage demo">
</p>

---

Back for a newer release? See the [upgrading guide](docs/notes/upgrading.md).
