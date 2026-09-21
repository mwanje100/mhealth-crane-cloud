.PHONY: build run test-normal test-partition simulate-client reproduce

build:
	cargo build --release

run-edge:
	cargo run --bin edge_node

simulate-client:
	cargo run --bin mobile_client

toggle-wan:
	curl -X GET http://127.0.0.1:3030/wan/toggle

reproduce: build
	@echo "Running reproducible simulation harness..."
	cargo test
