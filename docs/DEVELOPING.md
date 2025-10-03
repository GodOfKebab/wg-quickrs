# Developing wg-quickrs

Following are the scripts I use in my IDE to do my development.

---

## 1. WASM

Following will build the WASM target once.

```sh
# running directory: src/rust-wasm/
wasm-pack build --target web --out-dir ../web/pkg -- --features wasm
```

## 2. Vue/Frontend

Following will listen to the files and if I change them, automagically reload the web page.

```sh
# running directory: src/web/
npm run dev
```

## 3. Agent

Following will listen to the files and if I change them, recompile and rerun the agent.

Make sure the profile is `dev`, otherwise you will get CORS errors on the frontend side.
CORS is disabled when profile is release.

Also on the frontend development side, the default server prefix is `http://127.0.0.1:8080`.
So be sure to init your agent such that `http` is enabled and served at address `127.0.0.1` and port `8080`.

```sh
# make sure the dev config folder exists by running
#   mkdir -p ../.wg-quickrs
# initialize by running
# cargo run --profile dev -- --wg-quickrs-config-folder ../.wg-quickrs init
# running directory: src/
cargo watch -i web -x "run -- --wg-quickrs-config-folder ../.wg-quickrs agent run"
```

To test release profile.

```sh
# running directory: src/
cargo run --release -- --wg-quickrs-config-folder ../.wg-quickrs agent run
```

