#!/bin/sh

cargo build

mkdir -p output

for i in {1..7}; do
	sudo ../../target/debug/judge-client-3 test$i 0 .
done

sudo chmod 666 output/*
