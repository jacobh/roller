.PHONY: check build

build:
	wasm-pack build --target web && rollup ./main.js --format iife --file ./pkg/bundle.js

build-dev:
	wasm-pack build --dev --target web && rollup ./main.js --format iife --file ./pkg/bundle.js

check:
	cargo check --target=wasm32-unknown-unknown

