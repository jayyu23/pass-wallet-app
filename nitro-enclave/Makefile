# Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

ARCH := $(shell uname -m)
RUST_DIR := $(shell readlink -m $(shell dirname $(firstword $(MAKEFILE_LIST))))

DOWNLOAD_RUST := 0

.DEFAULT_GOAL := enclave

build:
ifeq ($(DOWNLOAD_RUST), 1)
	curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
	rustup target install ${ARCH}-unknown-linux-musl
endif
	cargo build --manifest-path=${RUST_DIR}/Cargo.toml --target=${ARCH}-unknown-linux-musl --release
	cp ${RUST_DIR}/target/${ARCH}-unknown-linux-musl/release/nitro-enclave ${RUST_DIR}


server: build
	docker build -t nitro-enclave-server -f Dockerfile.server .
	nitro-cli build-enclave --docker-uri nitro-enclave-server --output-file nitro_enclave_server.eif

client: build
	docker build -t nitro-enclave-client -f Dockerfile.client .
	nitro-cli build-enclave --docker-uri nitro-enclave-client --output-file nitro_enclave_client.eif

run-client: build
	./nitro-enclave client --cid 16 --port 7777

.PHONY: clean
clean:
	rm -rf ${RUST_DIR}/target ${RUST_DIR}/nitro_enclave_*.eif ${RUST_DIR}/server ${RUST_DIR}/client

enclave: server
	@echo "Starting enclave..."
	@ENCLAVE_ID_NEW=$$(nitro-cli run-enclave --eif-path nitro_enclave_server.eif --cpu-count 2 --memory 256 --debug-mode --enclave-cid 16 | grep -o 'i-[0-9a-f]*-enc[0-9a-f]*' | head -1) && \
	export ENCLAVE_ID=$$ENCLAVE_ID_NEW && \
	export ENCLAVE_CID=16 && \
	echo "export ENCLAVE_ID=$$ENCLAVE_ID_NEW && export ENCLAVE_CID=16"

http-server:
	cargo run --bin http-server

stop:
	sudo nitro-cli terminate-enclave --all

start: enclave
