.PHONY: dynamo
dynamo:
	- docker compose down --volumes
	docker compose up -d --wait dynamodb
	docker compose up aws-cli
	deno run --allow-net=localhost:8000 --allow-env --no-check ./setup/setup.ts

.PHONY: test
test:
	make dynamo
	cargo test -- --test-threads=1
	make dynamo
	cargo test --no-default-features --features aws-sdk -- --test-threads=1

.PHONY: lint
lint:
	cargo clippy --all-targets -- -D warnings
	cargo clippy --all-targets --features tracing -- -D warnings
	cargo clippy --all-targets --no-default-features --features rusoto_rustls -- -D warnings
	cargo clippy --all-targets --no-default-features --features aws-sdk -- -D warnings

.PHONY: check-deps
check-deps:
	cargo machete || echo
	cargo +nightly udeps --all-targets
	cargo +nightly udeps --all-targets --features tracing
	cargo +nightly udeps --all-targets --no-default-features --features rusoto_rustls
	cargo +nightly udeps --all-targets --no-default-features --features aws-sdk
