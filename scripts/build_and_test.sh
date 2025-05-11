#!/bin/bash

set -e
skip_build=false
detach=false
filtered_args=()

for arg in "$@"; do
  if [[ "$arg" == "--skip-build" ]]; then
    skip_build=true
  elif [[ "$arg" == "--detach" ]]; then
    detach=true
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

if [[ "$detach" == "true" ]]; then
  echo "Running tests with args: --detach --skip-build ${filtered_args[@]}"
  anchor test --detach --skip-build "${filtered_args[@]}"
else
  echo "Running tests with args: --skip-build ${filtered_args[@]}"
  anchor test --skip-build "${filtered_args[@]}"
fi
