# wg-quickrs quick-start guide for routers

Because `wg-quickrs` has a minimal disk-space and memory footprint, it can run on resource-constrained devices like home routers.
On idle, it consumes around **~15MB of disk space** (binary + conf.yml/tls files), **~10MB of RAM on idle and ~30MB of RAM at peak** (for <1 second when the server handles a new client).

Steps are very similar to [servers](./server.md).

## AsusWRT-Merlin

⚠️ Note: This is still a work in progress.
I have an ASUS router running Asuswrt-Merlin, and it fails due to routing issues.
In this state, handshake succeeds, but the client cannot connect to the internet.

Since `installer.sh` only looks for `apt-get` and `apk` package managers, if your router uses another package manager, you will need to install `resolvconf` (and maybe some other packages) manually.
For that, you can skip the dependency installation.
Additionally, if `/usr/local/bin` doesn't exist, you can install to user.

```shell
wget -q https://github.com/GodOfKebab/wg-quickrs/releases/latest/download/installer.sh
sh installer.sh --skip-deps --install-to user
```

TLS key generation might also take some time.
Otherwise, the web console and cli work fine.

To let `wg-quickrs` binary and config folder persist across reboots, you can move them to `jffs` partition:

```shell
mv ~/.wg-quickrs /jffs/wg-quickrs
mv ~/.local/bin/wg-quickrs /jffs/scripts/wg-quickrs

# Initialize the agent
/jffs/scripts/wg-quickrs --wg-quickrs-config-folder /jffs/wg-quickrs agent init

# Load WireGuard module
modprobe wireguard

# Run the agent
/jffs/scripts/wg-quickrs --wg-quickrs-config-folder /jffs/wg-quickrs agent run
```
