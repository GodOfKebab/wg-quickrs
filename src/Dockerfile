FROM --platform=linux/$BUILDARCH rust:1.89-slim AS rust-wasm-builder
WORKDIR /app/rust-wasm
COPY src/rust-wasm /app/rust-wasm
RUN cargo install wasm-pack
RUN wasm-pack build --target web --out-dir /app/web/pkg -- --features wasm

FROM --platform=linux/$BUILDARCH node:24-alpine AS node-builder
WORKDIR /app/web
COPY src/web .
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

COPY src/Cargo.toml /app/Cargo.toml
COPY src/rust-wasm /app/rust-wasm
COPY src/rust-agent /app/rust-agent
COPY src/rust-cli /app/rust-cli
COPY src/web/package.json /app/web/package.json
COPY .git /app/.git

# Setup target arch
ARG TARGETARCH
RUN case "$TARGETARCH" in \
        amd64) echo 'x86_64-unknown-linux-musl' > RUST_TARGET ;; \
        arm64) echo 'aarch64-unknown-linux-musl' > RUST_TARGET ;; \
        arm) echo 'armv7-unknown-linux-musleabihf' > RUST_TARGET ;; \
        *) echo "Unsupported architecture $TARGETARCH" >&2; exit 1 ;; \
    esac && \
    rustup target add $(cat RUST_TARGET)

# Copy web folder right before build for build optimization
COPY --from=node-builder /app/web/dist /app/web/dist
RUN cargo zigbuild --release --package wg-quickrs --bin wg-quickrs --target=$(cat RUST_TARGET) && \
    cp /app/target/$(cat RUST_TARGET)/release/wg-quickrs /app/wg-quickrs

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
exec /app/wg-quickrs --wg-quickrs-config-folder .wg-quickrs "$@"
EOF
COPY --from=rust-agent-builder /app/wg-quickrs /app/wg-quickrs
ENTRYPOINT ["/bin/bash", "/app/entrypoint.sh"]
#CMD ["tail", "-f", "/dev/null"]
