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

test.creds:
	cargo test -p potato-agent test_get_gemini_credentials_token -- --nocapture --test-threads=1