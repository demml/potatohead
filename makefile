.PHONY: build
format:
	cargo fmt --all

.PHONY: lints
lints:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test -- --nocapture --test-threads=1

test.prompt:
	cargo test -p potato-prompt -- --nocapture --test-threads=1

test.baked:
	cargo test -p baked-potato -- --nocapture --test-threads=1
