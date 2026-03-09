.PHONY: build watch serve all

build:
	cd compiler && cargo run
watch:
	cd compiler && cargo watch -w ../math -x run
serve:
	cd public && python3 -m http.server 8000
dev:
	@echo "Starting Atlas Development Suite..."
	make build
	make watch & make serve
