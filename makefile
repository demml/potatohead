.PHONY: build
format:
	cargo fmt --all

.PHONY: lints
lints:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test -- --nocapture --test-threads=1

test.baked:
	cargo test -p baked-potato test_mock_server -- --nocapture --test-threads=1