#!/bin/bash

function build_bpf() {
    cargo build-bpf --manifest-path=program/Cargo.toml --bpf-out-dir=dist/program
}

case $1 in
    "build-bpf")
	build_bpf
	;;
    "deploy")
	build_bpf
	solana program deploy dist/program/index_lib.so
	;;
    "client")
	(cd client/; cargo run --bin client ../dist/program/index_lib-keypair.json)
	;;
    "price_sender")
	(cd client/; cargo run --bin price_sender ../dist/program/index_lib-keypair.json 61d2f7f2-a726-4e92-84a2-6f622bbfb6cc)
	;;
    "clean")
	(cd program/; cargo clean)
	(cd client/; cargo clean)
	rm -rf dist/
	;;
    *)
	echo "usage: $0 build-bpf"
	;;
esac
