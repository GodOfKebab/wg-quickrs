# Developing wg-quickrs

Following are the scripts I use in my IDE to do my development.

---

## 1. WASM

Following will build the WASM the target once.

```sh
# running directory: src/
wasm-pack build wg-quickrs-web --target web --out-dir ../wg-quickrs-web/pkg -- --features wasm
```

## 2. Vue/Frontend

Following will listen to the files, and if I change them, automagically reload the web page.

```sh
# running directory: src/wg-quickrs-web/
npm run dev
```

## 3. Agent

Following will listen to the files and if I change them, recompile and rerun the agent.

Make sure the profile is `dev`, otherwise you will get CORS errors on the frontend side.
CORS is disabled when the profile is released.

Also on the frontend development side, the default server prefix is `http://127.0.0.1:8080`.
So be sure to init your agent such that `http` is enabled and served at address `127.0.0.1` and port `8080`.

```sh
# running directory: src/
# make sure the dev config folder exists by running
#   mkdir -p ../.wg-quickrs
# initialize by running
#   cargo run --profile dev -- --wg-quickrs-config-folder ../.wg-quickrs init
cargo watch -i wg-quickrs-web -x "run -- --wg-quickrs-config-folder ../.wg-quickrs agent run"
```

To test the release profile.

```sh
# running directory: src/
cargo run --release -- --wg-quickrs-config-folder ../.wg-quickrs agent run
```

# Testing wg-quickrs

There are both unit tests (rust) and functional tests (python for cli and api).

For the unit tests, run the following.

```sh
# running directory: src/
# generate wasm target
wasm-pack build wg-quickrs-web --target web --out-dir ../wg-quickrs-web/pkg -- --features wasm
# build frontend
cd ../wg-quickrs-web/
npm ci --omit=dev
npm run build
cd ..
# run unit tests
cargo test
```

For the functional tests, run the following.

```sh
# running directory: src/
# generate release target (this is what pytest will use)
cargo build --release
# install testing dependencies
cd ../tests
npm install -g @usebruno/cli
pip3 install -r tests/requirements.txt
sudo apt update && sudo apt install wireguard
# run functional tests
pytest .
```

