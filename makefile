.PHONY: dynamo test lint

dynamo:
	- docker rm -f dynamodb
	- docker stop dynamodb
	docker run --rm -d --name dynamodb -p 8000:8000 amazon/dynamodb-local:latest
	AWS_ACCESS_KEY_ID=dummy AWS_SECRET_ACCESS_KEY=dummy deno run --allow-net=localhost:8000 --allow-env --no-check ./setup/setup.ts

test:
	make dynamo
	cargo test -- --test-threads=1

lint:
	cargo clippy --all-targets -- -D warnings
