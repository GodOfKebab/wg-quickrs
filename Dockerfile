FROM --platform=linux/$BUILDARCH rust:1.89-slim AS rust-wasm-builder
WORKDIR /app/rust-wasm
COPY rust-wasm/ /app/rust-wasm
RUN cargo install wasm-pack
RUN wasm-pack build --target web --out-dir /app/web/pkg -- --features wasm

FROM --platform=linux/$BUILDARCH node:24-alpine AS node-builder
COPY --from=rust-wasm-builder /app/web/pkg /app/web/pkg
WORKDIR /app/web
COPY web/ .
RUN npm ci --production

FROM --platform=linux/$BUILDARCH rust:1.89-slim AS rust-agent-builder
COPY --from=node-builder /app/web/dist /app/web/dist
WORKDIR /app
ARG TARGETARCH
ENV ZIG_TARGET=${TARGETARCH}
ENV RUST_TARGET=${TARGETARCH}
RUN case "$TARGETARCH" in \
        amd64) export ZIG_TARGET=x86_64; export RUST_TARGET=x86_64-unknown-linux-musl ;; \
        arm64) export ZIG_TARGET=aarch64; export RUST_TARGET=aarch64-unknown-linux-musl ;; \
        arm) export ZIG_TARGET=arm; export RUST_TARGET=armv7-unknown-linux-musleabihf ;; \
        *) echo "Unsupported architecture $TARGETARCH" >&2; exit 1 ;; \
        esac

# Install Zig
RUN apt-get update && apt-get install -y curl xz-utils musl-dev cmake clang llvm-dev libclang-dev pkg-config  && \
    rm -rf /var/lib/apt/lists/* && \
    curl -L https://ziglang.org/download/0.15.1/zig-${ZIG_TARGET}-linux-0.15.1.tar.xz | tar -xJ && \
    mv zig-* /usr/local/zig && \
    ln -s /usr/local/zig/zig /usr/local/bin/zig && \
    cargo install cargo-zigbuild

COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY rust-wasm/ /app/rust-wasm
COPY rust-agent/ /app/rust-agent
COPY rust-cli/ /app/rust-cli
COPY web/package.json /app/web/package.json
RUN echo "Building for $RUST_TARGET" && \
    rustup target add $RUST_TARGET && \
    cargo zigbuild --release --package wg-rusteze --bin wg-rusteze --target=${RUST_TARGET} && \
    cp /app/target/${RUST_TARGET}/release/wg-rusteze /app/wg-rusteze

FROM alpine:3.22 AS runner
COPY --from=rust-agent-builder /app/wg-rusteze /app/wg-rusteze
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
ENTRYPOINT ["/bin/bash", "/app/entrypoint.sh"]
#CMD ["tail", "-f", "/dev/null"]
