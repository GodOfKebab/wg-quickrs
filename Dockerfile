FROM rust:1.89-slim AS rust-wasm-builder

WORKDIR /app/rust-wasm
COPY rust-wasm/ /app/rust-wasm
RUN cargo install wasm-pack
RUN wasm-pack build --target web --out-dir /app/web/pkg -- --features wasm

FROM node:24-alpine AS node-builder
WORKDIR /app/web
COPY --from=rust-wasm-builder /app/web/pkg /app/web/pkg
COPY web/ .
RUN npm ci --production

FROM rust:1.89-slim AS rust-agent-builder
WORKDIR /app

# Install musl cross-compile dependencies
RUN apk add --no-cache musl-dev gcc

COPY --from=node-builder /app/web/dist /app/web/dist
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY rust-wasm/ /app/rust-wasm
COPY rust-agent/ /app/rust-agent
COPY rust-cli/ /app/rust-cli
COPY web/package.json /app/web/package.json
RUN cargo build --bin wg-rusteze --profile release

FROM alpine:3.22 AS runner
WORKDIR /app

RUN apk add -U --no-cache wireguard-tools iptables
COPY --from=rust-agent-builder /app/target/release/wg-rusteze /app/wg-rusteze

RUN cat > /app/entrypoint.sh <<'EOF'
#!/bin/bash
# ensure PATH includes system sbin/bin
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

# create a dummy sudo if missing
if [ ! -x /usr/bin/sudo ]; then
  echo '#!/bin/sh' > /usr/bin/sudo
  echo 'exec "$@"' >> /usr/bin/sudo
  chmod +x /usr/bin/sudo
fi

# run the actual app
exec /app/wg-rusteze --wg-rusteze-config-folder .wg-rusteze "$@"
EOF

ENTRYPOINT ["/bin/bash", "/app/entrypoint.sh"]
#CMD ["tail", "-f", "/dev/null"]
