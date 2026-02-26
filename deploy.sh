#!/bin/bash

KEY="$HOME/.ssh/TestMac.pem"
HOST="ec2-user@YOUR_EC2_IP"

set -e

echo "Copying project..."
scp -i "$KEY" -r ./order-api "$HOST":~/order-api

echo "Building on EC2..."
ssh -i "$KEY" "$HOST" bash << 'EOF'
  pkill order-api || true
  cd ~/order-api
  source "$HOME/.cargo/env"
  cargo build --release
  nohup ./target/release/order-api > app.log 2>&1 &
  sleep 2
  tail -5 app.log
EOF
