dynamo:
	- docker rm -f dynamodb
	- docker stop dynamodb
	docker run --rm -d --name dynamodb -p 8000:8000 amazon/dynamodb-local:latest
	node setup

test:
	make dynamo
	cargo test -- --test-threads=1

lint:
	cargo clippy --all-targets -- -D warnings