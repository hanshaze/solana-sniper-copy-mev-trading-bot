#!/bin/bash

# 2) Set any necessary environment variables
export RUST_LOG=info
export BLOCK_ENGINE_URL="https://mainnet.block-engine.jito.wtf"

# Path to your approved Jito auth keypair
export AUTH_KEYPAIR="/home/ubuntu/shreds_sniper/shredstream-proxy/validator-keypair.json"

# Comma-separated Jito regions
export DESIRED_REGIONS="tokyo, frankfurt, amsterdam, ny, london, singapore, slc"

# Comma-separated list of destination IP:UDP_PORT entries
# (example: forward to localhost:8001 and a remote at 10.0.1.5:8002)
export DEST_IP_PORTS="127.0.0.1:8001"

# 3) Run the proxy binary
#    If you built with `cargo build --release`, the binary lives under `target/release/`
exec /home/ubuntu/shreds_sniper/shredstream-proxy/target/release/jito-shredstream-proxy shredstream \
     --block-engine-url "${BLOCK_ENGINE_URL}" \
     --auth-keypair "${AUTH_KEYPAIR}" \
     --desired-regions "${DESIRED_REGIONS}" \
     --dest-ip-ports "${DEST_IP_PORTS}" \
     --grpc-service-port 50051 \
