FROM --platform=linux/$BUILDARCH rust:1.90-slim AS rust-builder
WORKDIR /app

RUN apt update && apt install -y curl

COPY src/Cargo.toml /app/Cargo.toml
COPY src/Cargo.lock /app/Cargo.lock
COPY src/wg-quickrs /app/wg-quickrs
COPY src/wg-quickrs-cli /app/wg-quickrs-cli
COPY src/wg-quickrs-lib /app/wg-quickrs-lib

FROM rust-builder AS wg-quickrs-lib-builder
WORKDIR /app

ADD https://raw.githubusercontent.com/drager/wasm-pack/refs/heads/master/docs/_installer/init.sh init.sh
RUN bash init.sh
RUN wasm-pack build wg-quickrs-lib --target web --out-dir /app/pkg -- --locked

FROM --platform=linux/$BUILDARCH node:24-alpine AS node-builder
WORKDIR /app/wg-quickrs-web
COPY src/wg-quickrs-web .
RUN npm ci --omit=dev
COPY --from=wg-quickrs-lib-builder /app/pkg /app/wg-quickrs-web/pkg
RUN npm run build

FROM rust-builder AS wg-quickrs-builder
WORKDIR /app

# Install Zig
ARG BUILDARCH
RUN case "$BUILDARCH" in \
            amd64) echo 'x86_64' > ZIG_TARGET ;; \
            arm64) echo 'aarch64' > ZIG_TARGET ;; \
            arm) echo 'arm' > ZIG_TARGET ;; \
            *) echo "Unsupported architecture $BUILDARCH" >&2; exit 1 ;; \
            esac && \
    apt-get install -y xz-utils git cmake clang && \
    curl -L https://ziglang.org/download/0.15.1/zig-$(cat ZIG_TARGET)-linux-0.15.1.tar.xz | tar -xJ && \
    mv zig-* /usr/local/zig && \
    ln -s /usr/local/zig/zig /usr/local/bin/zig && \
    cargo install --locked cargo-zigbuild

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

# Copy wg-quickrs-web folder right before build for build optimization
COPY --from=node-builder /app/wg-quickrs-web/dist /app/wg-quickrs-web/dist
RUN cargo zigbuild --locked --release --package wg-quickrs --bin wg-quickrs --target=$(cat RUST_TARGET) --features docker && \
    mkdir /app/bin/ && cp /app/target/$(cat RUST_TARGET)/release/wg-quickrs /app/bin/wg-quickrs

FROM alpine:3.22 AS runner
WORKDIR /app
RUN apk add -U --no-cache wireguard-tools-wg openresolv iproute2 iptables
COPY --from=wg-quickrs-builder /app/bin/wg-quickrs /app/wg-quickrs
ENTRYPOINT ["/app/wg-quickrs", "--wg-quickrs-config-folder", ".wg-quickrs"]
#CMD ["tail", "-f", "/dev/null"]
