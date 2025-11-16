# Manual Testing with Vagrant Boxes

These are a collection of Vagrant boxes I use for testing docs and installer scripts.

---

## Vagrant Commands

```shell
# start vagrant box
vagrant up
vagrant ssh
# in vagrant shell

# Upload project folder to vagrant box (only for build)
vagrant rsync

# cleanup vagrant box
vagrant halt
vagrant destroy
```

---

## Build

Used to verify `docs/BUILDING.md`

```shell
cd wg-quickrs/src
# run-md.sh commands in docs/BUILDING.md

# optionally use the bridge interface as default gateway
default via <gateway ip of your bridge network> dev eth1
```


## Deployment

Used to verify `installer.sh`



