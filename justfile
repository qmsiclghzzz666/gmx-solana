IDL_OUT_DIR := "idl-out"
FEATURES := "cli,u128"

default: lint test test-programs

lint:
  cargo fmt --check
  cargo clippy --features {{FEATURES}}

test:
  cargo test --features {{FEATURES}}

test-programs:
  anchor test

build-idls:
  mkdir -p {{IDL_OUT_DIR}}
  anchor idl build -p gmsol_store -t {{IDL_OUT_DIR}}/gmsol_store.ts -o {{IDL_OUT_DIR}}/gmsol_store.json
