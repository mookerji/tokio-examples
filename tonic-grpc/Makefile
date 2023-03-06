PHONY: run
run:
	@docker compose up --build

PHONY: deps
deps:
	@brew install grpcurl
	@cargo install --locked cargo-outdated tokio-console

PHONY: format
format:
	@cargo fmt

PHONY: stop
stop:
	@docker compose down

PHONY: stop-all
stop-all:
	@docker stop $(docker ps -a -q)
