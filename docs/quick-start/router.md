# wg-quickrs quick-start guide for routers

⚠️ Note: This is still a work in progress.

Because `wg-quickrs` has a minimal disk-space and memory footprint, it can run on resource-constrained devices like home routers.
On idle, it consumes around **~15MB of disk space** (binary + conf.yml/tls files), **~10MB of RAM on idle and ~30MB of RAM at peak** (for <1 second when the server handles a new client).

Steps should be very similar to [servers](./server.md).

Since `installer.sh` only looks for `apt-get` and `apk` package managers, if your router uses another package manager, you will need to install `resolvconf` (and maybe some other packages) manually.
For that, you can skip the dependency installation.

```shell
sh installer.sh --skip-deps
```

I have an ASUS router running Asuswrt-Merlin, and it fails with dns resolution because `resolvconf` is not installed.
Otherwise, the web console and cli work fine.

