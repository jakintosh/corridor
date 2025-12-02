### Build matrix
# - Native (Linux host) is the primary path: `make build` / `make run`
# - WebAssembly is optional: `make wasm-bindgen` / `make wasm-serve`

CARGO       ?= cargo
PKG         := corridor

# Native build settings
HOST_TARGET ?= x86_64-unknown-linux-gnu
PROFILE     ?= release

# WASM build settings
WASM_TARGET   := wasm32-unknown-unknown
WASM_DIR      := target/$(WASM_TARGET)/$(PROFILE)
BINDGEN_OUT   := web/pkg
BINDGEN_FLAGS := --target web --no-typescript

.PHONY: build build-debug run clean \
        wasm-build wasm-bindgen wasm-clean wasm-serve

## Native (primary)
build:
	$(CARGO) build --target $(HOST_TARGET) --$(PROFILE)

build-debug:
	$(CARGO) build --target $(HOST_TARGET)

run: build
	$(CARGO) run --target $(HOST_TARGET) --$(PROFILE)

clean:
	$(CARGO) clean

## WebAssembly
wasm-build:
	$(CARGO) build --lib --target $(WASM_TARGET) --$(PROFILE)

wasm-bindgen: wasm-build
	wasm-bindgen $(WASM_DIR)/$(PKG).wasm --out-dir $(BINDGEN_OUT) $(BINDGEN_FLAGS)

wasm-clean:
	rm -rf $(BINDGEN_OUT)

wasm-serve: wasm-bindgen
	cd web && python3 -m http.server 8000
