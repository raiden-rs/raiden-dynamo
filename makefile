.PHONY: dynamo test lint

dynamo:
	- docker rm -f dynamodb
	- docker stop dynamodb
	docker run --rm -d --name dynamodb -p 8000:8000 amazon/dynamodb-local:latest
	deno run --allow-net=localhost:8000 --allow-env --no-check ./setup/setup.ts

test:
	make dynamo
	cargo test -- --test-threads=1

lint:
	cargo clippy --all-targets -- -D warnings
	cargo clippy --all-targets --no-default-features --features rustls -- -D warnings

.PHONY: check-deps
check-deps:
	cargo machete || echo
	cargo +nightly udeps --all-targets
	cargo +nightly udeps --all-targets --no-default-features --features rustls
