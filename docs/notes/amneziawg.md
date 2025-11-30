# AmneziaWG Setup Guide

Since `wg-quickrs` allows you to specify which `wg` and which userspace implementation to use, it is super easy to set up [AmneziaWG](https://docs.amnezia.org/documentation/amnezia-wg/).
But you still need to install it before you can use it.

ℹ️ Note 1: `wg-quickrs agent init` command will seamlessly use `AmneziaWG` if it is installed.
If not, it will fall back to the default `wg` implementation.
So you should follow this guide, then run `wg-quickrs agent init`.

⚠️ Note 2: You need to use `AmneziaWG` app, not the `AmneziaVPN` on the client.

## 1. `AmneziaWG` Installation 

### Use Docker (recommended)

The `awg` and `amneziawg-go` binaries are already included in the Docker image.
By default, the network is not obfuscated to be backwards compatible with the official WireGuard app.
To enable obfuscation, either
- follow the instructions in the [docker-compose.yml](../../docker-compose.yml)
- or on the web console, click on the gear icon in the top left corner > Amnezia Settings, then make sure the button "Enable AmneziaWG Obfuscation" is checked. Then "Save Configuration".

⚠️ Note: When you toggle AmneziaWG obfuscation, your clients' configuration will be updated, you will need to re-download .conf files or re-scan the QR.

### Build and Install `AmneziaWG`

Install the following packages:

```shell
# Debian/Ubuntu
sudo apt update && sudo apt install -y git build-essential golang-go
# Alpine
# doas apk add -U --no-cache git make build-base linux-headers go
```

### Build and Install `amneziawg-tools`

To build and install `awg(8)`, run the following commands:

```shell
git clone https://github.com/amnezia-vpn/amneziawg-tools.git
cd amneziawg-tools/src
make
sudo make install
# OR
# doas make install
```

### Build and Install `amneziawg-go`

For installing the kernel module, see [here](https://github.com/amnezia-vpn/amneziawg-linux-kernel-module).
Since the kernel module is harder to set up for alpine, Docker images and this guide will use the userspace implementation.

To build the userspace implementation, run the following commands:

```shell
git clone https://github.com/amnezia-vpn/amneziawg-go
cd amneziawg-go
make
sudo make install
# OR
# doas make install
```

## 2. `wg-quickrs` setup

Make sure that the conf.yml file looks like this:

```yaml
# ...
agent:
  # ...
  vpn:
    enabled: true
    port: 51820
    wg: /usr/bin/awg  # replace this with the path to your `awg` binary (which awg)
    wg_userspace:
      enabled: true
      binary: /usr/bin/amneziawg-go  # replace this with the path to your `amneziawg-go` binary (which amneziawg-go)
network:
  # ...
  amnezia_parameters:
    enabled: true  # make sure this is set to true, otherwise there will be no obfuscation
    # ...
```

See [conf.yml schema](./schema.md) for more details and [official notes](https://github.com/amnezia-vpn/amneziawg-linux-kernel-module?tab=readme-ov-file#configuration) for obfuscation parameter ranges.
