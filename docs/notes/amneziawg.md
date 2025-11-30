# AmneziaWG Setup Guide

Since `wg-quickrs` allows you to specify which `wg` and which userspace implementation to use, it is super easy to set up [AmneziaWG](https://docs.amnezia.org/documentation/amnezia-wg/).

⚠️ Note 1: You need to use `AmneziaWG` app, not the `AmneziaVPN` on the client.

## `wg-quickrs` setup

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


## Use Docker (recommended)

The `awg` and `amneziawg-go` binaries are already included in the Docker image.
So follow the instructions in the [docker-compose.yml](../../docker-compose.yml) to enable obfuscation.


## Build and Install AmneziaWG on Alpine (for servers and Docker containers)

⚠️ Note 2: The installation guide here is intended for alpine hosts.

Install the following packages:

```shell
apk add -U --no-cache git make go linux-headers
```

### Build and Install `amneziawg-tools`

To build and install `awg(8)`, run the following commands:

```shell
git clone https://github.com/amnezia-vpn/amneziawg-tools.git
cd amneziawg-tools/src
make
make install
```

### Build and Install `amneziawg-go`

For installing the kernel module, see [here](https://github.com/amnezia-vpn/amneziawg-linux-kernel-module).

Since the kernel module is not available for alpine, Docker images and this guide will use the userspace implementation.

To build the userspace implementation, run the following commands:

```shell
git clone https://github.com/amnezia-vpn/amneziawg-go
cd amneziawg-go
make
make install
```
