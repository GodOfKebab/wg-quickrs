FROM --platform=linux/$BUILDARCH rust:1.89-slim AS rust-wasm-builder
WORKDIR /app/rust-wasm
COPY rust-wasm/ /app/rust-wasm
RUN cargo install wasm-pack
RUN wasm-pack build --target web --out-dir /app/web/pkg -- --features wasm

FROM --platform=linux/$BUILDARCH node:24-alpine AS node-builder
WORKDIR /app/web
COPY web/ .
RUN npm ci --omit=dev
COPY --from=rust-wasm-builder /app/web/pkg /app/web/pkg
RUN npm run build

FROM --platform=linux/$BUILDARCH rust:1.89-slim AS rust-agent-builder
WORKDIR /app

# Install Zig
ARG BUILDARCH
RUN case "$BUILDARCH" in \
            amd64) echo 'x86_64' > ZIG_TARGET ;; \
            arm64) echo 'aarch64' > ZIG_TARGET ;; \
            arm) echo 'arm' > ZIG_TARGET ;; \
            *) echo "Unsupported architecture $BUILDARCH" >&2; exit 1 ;; \
            esac && \
    apt-get update && apt-get install -y curl xz-utils git musl-dev cmake clang llvm-dev libclang-dev pkg-config && \
    rm -rf /var/lib/apt/lists/* && \
    curl -L https://ziglang.org/download/0.15.1/zig-$(cat ZIG_TARGET)-linux-0.15.1.tar.xz | tar -xJ && \
    mv zig-* /usr/local/zig && \
    ln -s /usr/local/zig/zig /usr/local/bin/zig && \
    cargo install cargo-zigbuild

COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY rust-wasm/ /app/rust-wasm
COPY rust-agent/ /app/rust-agent
COPY rust-cli/ /app/rust-cli
COPY web/package.json /app/web/package.json
COPY .git /app/.git

ARG TARGETARCH
COPY --from=node-builder /app/web/dist /app/web/dist
RUN case "$TARGETARCH" in \
                amd64) echo 'x86_64-unknown-linux-musl' > RUST_TARGET ;; \
                arm64) echo 'aarch64-unknown-linux-musl' > RUST_TARGET ;; \
                arm) echo 'armv7-unknown-linux-musleabihf' > RUST_TARGET ;; \
                *) echo "Unsupported architecture $TARGETARCH" >&2; exit 1 ;; \
                esac && \
    rustup target add $(cat RUST_TARGET) && \
    cargo zigbuild --release --package wg-rusteze --bin wg-rusteze --target=$(cat RUST_TARGET) && \
    cp /app/target/$(cat RUST_TARGET)/release/wg-rusteze /app/wg-rusteze

FROM alpine:3.22 AS runner
WORKDIR /app
RUN apk add -U --no-cache wireguard-tools iptables
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
COPY --from=rust-agent-builder /app/wg-rusteze /app/wg-rusteze
ENTRYPOINT ["/bin/bash", "/app/entrypoint.sh"]
#CMD ["tail", "-f", "/dev/null"]
