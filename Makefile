.PHONY: build test test-features lint fmt clean check-wasm-size

build:
	cargo build --target wasm32v1-none --release

test:
	cargo test --features testutils

test-features:
	cargo test --features testutils --all-features

lint:
	cargo clippy --all-targets -- --deny warnings

fmt:
	cargo fmt --check

clean:
	cargo clean

check-wasm-size:
	@ls -la target/wasm32v1-none/release/*.wasm 2>/dev/null || echo "WASM not built yet. Run 'make build' first."
	@for wasm in target/wasm32v1-none/release/*.wasm; do \
		size=$$(stat -c%s "$$wasm" 2>/dev/null || stat -f%z "$$wasm" 2>/dev/null); \
		echo "$$wasm: $$size bytes"; \
		if [ "$$size" -gt 65536 ]; then \
			echo "WARNING: $$wasm exceeds 64KB limit!"; \
		else \
			echo "OK: within 64KB limit"; \
		fi; \
	done

all: build lint fmt test check-wasm-size
