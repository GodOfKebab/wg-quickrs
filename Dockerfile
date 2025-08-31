FROM docker.io/library/rust:1.89.0-trixie AS rust-wasm-builder

WORKDIR /app/rust-wasm
COPY rust-wasm/ /app/rust-wasm
RUN cargo install wasm-pack
RUN wasm-pack build --target web --out-dir /app/web/pkg -- --features wasm

FROM docker.io/library/node:24-alpine AS node-builder
COPY --from=rust-wasm-builder /app/web/pkg /app/web/pkg

WORKDIR /app/web
COPY web/ /app/web
RUN npm ci --production

FROM docker.io/library/rust:1.89.0-trixie AS rust-agent-builder
COPY --from=node-builder /app/web/dist /app/web/dist

WORKDIR /app
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY rust-wasm/ /app/rust-wasm
COPY rust-agent/ /app/rust-agent
COPY web/package.json /app/web/package.json
RUN cargo build --bin wg-rusteze --profile release

FROM docker.io/library/debian:trixie-slim AS initializer
COPY --from=rust-agent-builder /app/target/release/wg-rusteze /app/wg-rusteze
WORKDIR /app

RUN apt-get update && apt-get install -y wireguard-tools

CMD /app/wg-rusteze init --no-prompt true \
  --network-identifier "$NETWORK_IDENTIFIER" \
  --network-subnet "$NETWORK_SUBNET" \
  --agent-peer-name "$AGENT_PEER_NAME" \
  --agent-public-address "$AGENT_PUBLIC_ADDRESS" \
  --agent-web-port "$AGENT_WEB_PORT" \
  --agent-vpn-port "$AGENT_VPN_PORT" \
  --agent-internal-vpn-address "$AGENT_INTERNAL_VPN_ADDRESS" \
  --agent-use-tls "$AGENT_USE_TLS" \
  --agent-enable-web-password "$AGENT_ENABLE_WEB_PASSWORD" \
  --agent-web-password "$AGENT_WEB_PASSWORD" \
  --agent-enable-dns "$AGENT_ENABLE_DNS" \
  --agent-dns-server "$AGENT_DNS_SERVER" \
  --agent-enable-mtu "$AGENT_ENABLE_MTU" \
  --agent-mtu-value "$AGENT_MTU_VALUE" \
  --agent-enable-script-pre-up "$AGENT_ENABLE_SCRIPT_PRE_UP" \
  --agent-script-pre-up-line "$AGENT_SCRIPT_PRE_UP_LINE" \
  --agent-enable-script-post-up "$AGENT_ENABLE_SCRIPT_POST_UP" \
  --agent-script-post-up-line "$AGENT_SCRIPT_POST_UP_LINE" \
  --agent-enable-script-pre-down "$AGENT_ENABLE_SCRIPT_PRE_DOWN" \
  --agent-script-pre-down-line "$AGENT_SCRIPT_PRE_DOWN_LINE" \
  --agent-enable-script-post-down "$AGENT_ENABLE_SCRIPT_POST_DOWN" \
  --agent-script-post-down-line "$AGENT_SCRIPT_POST_DOWN_LINE" \
  --default-enable-dns "$DEFAULT_ENABLE_DNS" \
  --default-dns-server "$DEFAULT_DNS_SERVER" \
  --default-enable-mtu "$DEFAULT_ENABLE_MTU" \
  --default-mtu-value "$DEFAULT_MTU_VALUE" \
  --default-enable-script-pre-up "$DEFAULT_ENABLE_SCRIPT_PRE_UP" \
  --default-script-pre-up-line "$DEFAULT_SCRIPT_PRE_UP_LINE" \
  --default-enable-script-post-up "$DEFAULT_ENABLE_SCRIPT_POST_UP" \
  --default-script-post-up-line "$DEFAULT_SCRIPT_POST_UP_LINE" \
  --default-enable-script-pre-down "$DEFAULT_ENABLE_SCRIPT_PRE_DOWN" \
  --default-script-pre-down-line "$DEFAULT_SCRIPT_PRE_DOWN_LINE" \
  --default-enable-script-post-down "$DEFAULT_ENABLE_SCRIPT_POST_DOWN" \
  --default-script-post-down-line "$DEFAULT_SCRIPT_POST_DOWN_LINE" \
  --default-enable-persistent-keepalive "$DEFAULT_ENABLE_PERSISTENT_KEEPALIVE" \
  --default-persistent-keepalive-period "$DEFAULT_PERSISTENT_KEEPALIVE_PERIOD"

FROM docker.io/library/debian:trixie-slim AS runner
COPY --from=rust-agent-builder /app/target/release/wg-rusteze /app/wg-rusteze
WORKDIR /app

#CMD ["tail", "-f", "/dev/null"]
ENTRYPOINT ["/app/wg-rusteze"]
