PHONY: run
run:
	@docker compose up --build

PHONY: deps
deps:
	@brew install grpcurl
	@cargo install --locked cargo-outdated tokio-console
