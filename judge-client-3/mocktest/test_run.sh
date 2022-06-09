#!/bin/bash

set +e

cargo build

mkdir -p output

sudo install -v -d -m755 /run/systemd/system
cat << EOF | sudo tee /run/systemd/system/opoj-42.slice > /dev/null
[Slice]
CPUQuota=100%
AllowedCPUs=0
TasksMax=32
EOF

sudo systemctl stop opoj-42.slice
sudo systemctl daemon-reload

for i in {1..10}; do
	sudo ../../target/debug/judge-client-3 test$i 42 .
done

sudo chmod 666 output/*
