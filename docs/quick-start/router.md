# wg-quickrs quick-start guide for routers

⚠️ Note: This is still a work in progress.

Steps should be very similar to [servers](./server.md).

I have an ASUS router running Asuswrt-Merlin, and it fails with dns resolution because `resolvconf` is not installed.
Otherwise, it works fine, but most of the functionality is not there without the DNS settings.

Since `installer.sh` only looks for `apt-get` and `apk` package managers, if your router uses another package manager, you will need to install `resolvconf` manually.
For that, you can skip the dependency installation.

```shell
sh installer.sh --skip-deps
```


