.PHONY: build
build:
	wasm-pack build --target web --release; rm -r ./frontend/pkg; mv ./pkg ./frontend/pkg

.PHONY: start
start:
	cd frontend && bun run dev

.PHONY: fmt
fmt:
	cargo fmt
	cd frontend && bun run prettier