.PHONY: dynamo
dynamo:
	- docker rm -f dynamodb
	- docker stop dynamodb
	docker run --rm -d --name dynamodb -p 8000:8000 amazon/dynamodb-local:latest
	deno run --allow-net=localhost:8000 --allow-env --no-check ./setup/setup.ts

.PHONY: test
test:
	make dynamo
	cargo test -- --test-threads=1

.PHONY: lint
lint:
	cargo clippy --all-targets -- -D warnings
	cargo clippy --all-targets --no-default-features --features rustls -- -D warnings
	cargo clippy --all-targets --features tracing -- -D warnings

.PHONY: check-deps
check-deps:
	cargo machete || echo
	cargo +nightly udeps --all-targets
	cargo +nightly udeps --all-targets --no-default-features --features rustls
	cargo +nightly udeps --all-targets --features tracing
