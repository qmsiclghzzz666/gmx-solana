#!/bin/bash

set -e
skip_build=false
filtered_args=()

for arg in "$@"; do
  if [[ "$arg" == "--skip-build" ]]; then
    skip_build=true
  else
    filtered_args+=("$arg")
  fi
done

if [[ "$skip_build" == "true" ]]; then
  echo "Skipping build step"
else
  echo "Running build step with args: --no-idl ${filtered_args[@]}"
  anchor build --no-idl ${filtered_args[@]}
fi

echo "Running tests with args: --skip-build ${filtered_args[@]}"
anchor test --skip-build "${filtered_args[@]}"
