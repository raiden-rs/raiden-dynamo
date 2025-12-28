export AWS_ACCESS_KEY_ID := dummy
export AWS_SECRET_ACCESS_KEY := dummy

.PHONY: dynamo
dynamo:
	docker compose down --volumes
	docker compose up -d --wait dynamodb
	docker compose up aws-cli
	deno run --allow-net=localhost:8000 --allow-env --allow-sys --no-check ./setup/setup.ts

.PHONY: test
test:
	make dynamo
	cargo test --no-default-features --features aws-sdk -- --test-threads=1
	make dynamo
	cargo test --no-default-features --features rusoto -- --test-threads=1

.PHONY: lint
lint:
	cargo clippy --all-targets --no-default-features --features aws-sdk -- -D warnings
	cargo clippy --all-targets --no-default-features --features rusoto -- -D warnings
	cargo clippy --all-targets --no-default-features --features rusoto_rustls -- -D warnings
	cargo clippy --all-targets --features tracing -- -D warnings

.PHONY: check-deps
check-deps:
	cargo machete
	cargo +nightly udeps --all-targets --no-default-features --features aws-sdk
	cargo +nightly udeps --all-targets --no-default-features --features rusoto
	cargo +nightly udeps --all-targets --no-default-features --features rusoto_rustls
	cargo +nightly udeps --all-targets --features tracing

.PHONY: licenses
licenses:
	cargo bundle-licenses --format toml --output THIRDPARTY.toml

.PHONY: check-licenses
check-licenses:
	RUST_LOG=error cargo bundle-licenses --format toml --output __CHECK --previous THIRDPARTY.toml --check-previous
	rm __CHECK || true
