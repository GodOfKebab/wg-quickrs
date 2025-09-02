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

FROM rust:1.89-alpine3.22 AS rust-agent-builder
WORKDIR /app

# Install musl cross-compile dependencies
RUN apk add --no-cache musl-dev gcc

COPY --from=node-builder /app/web/dist /app/web/dist
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY rust-wasm/ /app/rust-wasm
COPY rust-agent/ /app/rust-agent
COPY web/package.json /app/web/package.json
RUN cargo build --bin wg-rusteze --profile release

FROM alpine:3.22 AS runner
WORKDIR /app

RUN apk add -U --no-cache wireguard-tools
COPY --from=rust-agent-builder /app/target/release/wg-rusteze /app/wg-rusteze

#CMD ["tail", "-f", "/dev/null"]

FROM runner AS initializer
CMD /app/wg-rusteze init --no-prompt true \
  --network-identifier "$NETWORK_IDENTIFIER" \
  --network-subnet "$NETWORK_SUBNET" \
  --agent-peer-name "$AGENT_PEER_NAME" \
  --agent-local-address "$AGENT_LOCAL_ADDRESS" \
  --agent-local-enable-web-http "$AGENT_LOCAL_ENABLE_WEB_HTTP" \
  --agent-local-web-http-port "$AGENT_LOCAL_WEB_HTTP_PORT" \
  --agent-local-enable-web-https "$AGENT_LOCAL_ENABLE_WEB_HTTPS" \
  --agent-local-web-https-port "$AGENT_LOCAL_WEB_HTTPS_PORT" \
  --agent-local-web-https-tls-cert "$AGENT_LOCAL_WEB_HTTPS_TLS_CERT" \
  --agent-local-web-https-tls-key "$AGENT_LOCAL_WEB_HTTPS_TLS_KEY" \
  --agent-local-enable-vpn "$AGENT_LOCAL_ENABLE_VPN" \
  --agent-local-vpn-port "$AGENT_LOCAL_VPN_PORT" \
  --agent-public-address "$AGENT_PUBLIC_ADDRESS" \
  --agent-public-vpn-port "$AGENT_PUBLIC_VPN_PORT" \
  --agent-internal-vpn-address "$AGENT_INTERNAL_VPN_ADDRESS" \
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
