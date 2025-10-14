FROM --platform=linux/$BUILDARCH rust:1.89-slim AS wg-quickrs-wasm-builder
WORKDIR /app/wg-quickrs-wasm
COPY src/wg-quickrs-wasm /app/wg-quickrs-wasm
RUN cargo install wasm-pack
RUN wasm-pack build --target web --out-dir /app/pkg -- --features wasm

FROM --platform=linux/$BUILDARCH node:24-alpine AS node-builder
WORKDIR /app/wg-quickrs-web
COPY src/wg-quickrs-web .
RUN npm ci --omit=dev
COPY --from=wg-quickrs-wasm-builder /app/pkg /app/wg-quickrs-web/pkg
RUN npm run build

FROM --platform=linux/$BUILDARCH rust:1.89-slim AS wg-quickrs-builder
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

COPY src/Cargo.lock /app/Cargo.lock
COPY src/Cargo.toml /app/Cargo.toml
COPY src/wg-quickrs-wasm /app/wg-quickrs-wasm
COPY src/wg-quickrs /app/wg-quickrs
COPY src/wg-quickrs-cli /app/wg-quickrs-cli
COPY .git /app/.git

# Setup target arch
ARG TARGETARCH
RUN case "$TARGETARCH" in \
        amd64) echo 'x86_64-unknown-linux-musl' > RUST_TARGET ;; \
        arm64) echo 'aarch64-unknown-linux-musl' > RUST_TARGET ;; \
        arm) echo 'armv7-unknown-linux-musleabihf' > RUST_TARGET ;; \
        *) echo "Unsupported architecture $TARGETARCH" >&2; exit 1 ;; \
    esac && \
    rustup target add $(cat RUST_TARGET) && \
    cargo fetch --locked --target=$(cat RUST_TARGET)

# Copy wg-quickrs-web folder right before build for build optimization
COPY --from=node-builder /app/wg-quickrs-web/dist /app/wg-quickrs-web/dist
RUN cargo zigbuild --locked --release --package wg-quickrs --bin wg-quickrs --target=$(cat RUST_TARGET) --features docker && \
    mkdir /app/bin/ && cp /app/target/$(cat RUST_TARGET)/release/wg-quickrs /app/bin/wg-quickrs

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
COPY --from=wg-quickrs-builder /app/bin/wg-quickrs /app/wg-quickrs
ENTRYPOINT ["/bin/bash", "/app/entrypoint.sh"]
#CMD ["tail", "-f", "/dev/null"]
