# Manual Testing with Vagrant Boxes

These are a collection of Vagrant boxes I use for testing docs and installer scripts.

---

## Vagrant Commands

```shell
# start vagrant box
vagrant up
vagrant ssh
# in vagrant shell

# Upload synced folders to vagrant box (build and deployment boxes only)
vagrant rsync
# Upload synced files to vagrant box (deployment box only)
vagrant provision

# cleanup vagrant box
vagrant halt
vagrant destroy
```

---

## Build

Used to verify `docs/BUILDING.md`

```shell
# on the vm at ~/
cd wg-quickrs/src
# run-md.sh commands in docs/BUILDING.md

# optionally use the bridge interface as default gateway
default via <gateway ip of your bridge network> dev eth1
```


## Deployment

Used to verify `installer.sh`

```shell
# on the host machine at wg-quickrs/src/
# export RUST_TARGET=x86_64-unknown-linux-musl
# export RUST_TARGET=armv7-unknown-linux-musleabihf
export RUST_TARGET=aarch64-unknown-linux-musl
sh run-md.sh ../docs/BUILDING.md run-zig-build
sh run-md.sh ../docs/BUILDING.md create-a-distribution
```

```shell
# on the host machine at wg-quickrs/tests/vagrant/deployment/
# to upload dist/ to vagrant box
vagrant rsync

# or to upload wg-quickrs/installer.sh to vagrant box
vagrant provision
```

```shell
# on the vm at ~/
sh installer.sh --dist-tarball dist/wg-quickrs-aarch64-unknown-linux-musl.tar.gz
```

