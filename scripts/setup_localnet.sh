#!/bin/bash

if [ -z "${GMSOL_KEEPER}" ]; then
    echo "Error: GMSOL_KEEPER is not set" >&2
    exit 1
else
    echo "GMSOL_KEEPER is set to: $GMSOL_KEEPER"
fi

if [ -z "${GMSOL_ORACLE_SEED}" ]; then
    echo "Error: GMSOL_ORACLE_SEED is not set" >&2
    exit 1
else
    echo "GMSOL_ORACLE_SEED is set to: $GMSOL_ORACLE_SEED"
fi

if [ -z "${GMSOL_TOKENS}" ]; then
    echo "Error: GMSOL_TOKENS is not set" >&2
    exit 1
else
    echo "GMSOL_TOKENS is set to: $GMSOL_TOKENS"
fi

if [ -z "${GMSOL_MARKETS}" ]; then
    echo "Error: GMSOL_MARKETS is not set" >&2
    exit 1
else
    echo "GMSOL_MARKETS is set to: $GMSOL_MARKETS"
fi

if [ -z "${GMSOL_MARKET_CONFIGS}" ]; then
    echo "Error: GMSOL_MARKET_CONFIGS is not set" >&2
    exit 1
else
    echo "GMSOL_MARKET_CONFIGS is set to: $GMSOL_MARKET_CONFIGS"
fi

if [ -z "${LOCALNET_USDG_KEYPAIR}" ]; then
    echo "Error: LOCALNET_USDG_KEYPAIR is not set" >&2
    exit 1
else
    echo "LOCALNET_USDG_KEYPAIR is set to: $LOCALNET_USDG_KEYPAIR"
fi

export KEEPER_ADDRESS=$(solana-keygen pubkey $GMSOL_KEEPER)
solana -ul airdrop 10000 $KEEPER_ADDRESS
solana -ul airdrop 1 11111111111111111111111111111112
solana -ul airdrop 1 11111111111111111111111111111113

export USDG=$(solana-keygen pubkey $LOCALNET_USDG_KEYPAIR)
spl-token -ul create-token $LOCALNET_USDG_KEYPAIR --decimals 6
spl-token -ul create-account $USDG
spl-token -ul mint $USDG 1000

cargo gmsol -ul other init-mock-chainlink-verifier

export STORE=$(cargo gmsol -ul admin create-store)
cargo gmsol -ul admin init-roles --market-keeper $KEEPER_ADDRESS --order-keeper $KEEPER_ADDRESS

cargo gmsol -ul -w $GMSOL_KEEPER market init-gt \
    3000000000 \
    10000000000 \
    30000000000 \
    100000000000 \
    300000000000 \
    1000000000000 \
    3000000000000 \
    10000000000000 \
    30000000000000

cargo gmsol -ul -w $GMSOL_KEEPER market set-order-fee-discount-factors \
    0 \
    4000000000000000000 \
    6000000000000000000 \
    8000000000000000000 \
    10000000000000000000 \
    12000000000000000000 \
    14000000000000000000 \
    16000000000000000000 \
    18000000000000000000 \
    20000000000000000000

export TOKEN_MAP=$(cargo gmsol -ul market create-token-map)
export ORACLE=$(cargo gmsol -ul market init-oracle --seed $GMSOL_ORACLE_SEED)
cargo gmsol -ul -w $GMSOL_KEEPER market insert-token-configs $GMSOL_TOKENS --token-map $TOKEN_MAP --set-token-map
cargo gmsol -ul -w $GMSOL_KEEPER market create-markets $GMSOL_MARKETS --enable
cargo gmsol -ul -w $GMSOL_KEEPER market update-configs $GMSOL_MARKET_CONFIGS

cargo gmsol -ul -w $GMSOL_KEEPER market toggle-gt-minting 6bVH9Hu8TT9pjknUKcoJTiQadjzWYa1Zvvb2V3UhrRvU --enable
cargo gmsol -ul -w $GMSOL_KEEPER market toggle-gt-minting 9ryWpDKcgTr44jSWpu3eGBx64Vw3BAmJAgs7TGUmTLRn --enable

export COMMON_ALT=$(cargo gmsol -ul alt extend --init common $ORACLE)
export MARKET_ALT=$(cargo gmsol -ul alt extend --init market)

echo "STORE: $STORE"
echo "ORACLE: $ORACLE"
echo "USDG: $USDG"
echo "COMMON_ALT: $COMMON_ALT"
echo "MARKET_ALT: $MARKET_ALT"
