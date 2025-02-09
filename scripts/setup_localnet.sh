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

if [ -z "${LOCALNET_BTC_KEYPAIR}" ]; then
    echo "Error: LOCALNET_BTC_KEYPAIR is not set" >&2
    exit 1
else
    echo "LOCALNET_BTC_KEYPAIR is set to: $LOCALNET_BTC_KEYPAIR"
fi

if [ -z "${GMSOL_TIME_WINDOW}" ]; then
    echo "Error: GMSOL_TIME_WINDOW is not set" >&2
    exit 1
else
    echo "GMSOL_TIME_WINDOW is set to: $GMSOL_TIME_WINDOW"
fi

export CLUSTER=localnet
export STORE_PROGRAM_ID="Gmso1uvJnLbawvw7yezdfCDcPydwW2s2iqG3w6MDucLo"

export KEEPER_ADDRESS=$(solana-keygen pubkey $GMSOL_KEEPER)
solana -ul airdrop 10000 $KEEPER_ADDRESS
solana -ul airdrop 1 11111111111111111111111111111112
solana -ul airdrop 1 11111111111111111111111111111113

export USDG=$(solana-keygen pubkey $LOCALNET_USDG_KEYPAIR)
spl-token -ul create-token $LOCALNET_USDG_KEYPAIR --decimals 6
spl-token -ul create-account $USDG
spl-token -ul mint $USDG 1000000000000

export BTC=$(solana-keygen pubkey $LOCALNET_BTC_KEYPAIR)
spl-token -ul create-token $LOCALNET_BTC_KEYPAIR --decimals 8
spl-token -ul create-account $BTC
spl-token -ul mint $BTC 1000000000

export WSOL="So11111111111111111111111111111111111111112"
export SOL="11111111111111111111111111111111"

cargo gmsol other init-mock-chainlink-verifier

export ADDRESS=$(solana address)

export STORE=$(cargo gmsol admin create-store)

export RECEIVER=$(cargo gmsol treasury receiver)

cargo gmsol admin transfer-receiver $RECEIVER --confirm

export CONFIG=$(cargo gmsol treasury init-config)

cargo gmsol admin init-roles \
    --market-keeper $KEEPER_ADDRESS \
    --order-keeper $KEEPER_ADDRESS \
    --treasury-admin $KEEPER_ADDRESS \
    --treasury-withdrawer $KEEPER_ADDRESS \
    --treasury-keeper $KEEPER_ADDRESS \
    --timelock-admin $ADDRESS \
    --allow-multiple-transactions

export TREASURY=$(cargo gmsol -w $GMSOL_KEEPER treasury init-treasury 0)
cargo gmsol -w $GMSOL_KEEPER treasury set-treasury $TREASURY

cargo gmsol -w $GMSOL_KEEPER treasury insert-token $WSOL
cargo gmsol -w $GMSOL_KEEPER treasury toggle-token-flag $WSOL allow_deposit --enable
cargo gmsol -w $GMSOL_KEEPER treasury toggle-token-flag $WSOL allow_withdrawal --enable
cargo gmsol -w $GMSOL_KEEPER treasury insert-token $USDG
cargo gmsol -w $GMSOL_KEEPER treasury toggle-token-flag $USDG allow_deposit --enable
cargo gmsol -w $GMSOL_KEEPER treasury toggle-token-flag $USDG allow_withdrawal --enable

cargo gmsol -w $GMSOL_KEEPER market init-gt \
    -c 100000000000 \
    --grow-factor 102100000000000000000 \
    --grow-step 2100000000000 \
    6000000000 \
    20000000000 \
    60000000000 \
    200000000000 \
    600000000000 \
    2000000000000 \
    6000000000000 \
    20000000000000 \
    60000000000000

cargo gmsol admin grant-role $KEEPER_ADDRESS GT_CONTROLLER
cargo gmsol -w $GMSOL_KEEPER gt set-exchange-time-window $GMSOL_TIME_WINDOW
cargo gmsol admin revoke-role $KEEPER_ADDRESS GT_CONTROLLER

cargo gmsol -w $GMSOL_KEEPER market set-order-fee-discount-factors \
    0 \
    2000000000000000000 \
    3000000000000000000 \
    4000000000000000000 \
    5000000000000000000 \
    6000000000000000000 \
    7000000000000000000 \
    8000000000000000000 \
    9000000000000000000 \
    10000000000000000000

cargo gmsol -w $GMSOL_KEEPER treasury set-referral-reward \
    0 \
    2000000000000000000 \
    3000000000000000000 \
    4000000000000000000 \
    5000000000000000000 \
    6000000000000000000 \
    7000000000000000000 \
    8000000000000000000 \
    9000000000000000000 \
    10000000000000000000

cargo gmsol -w $GMSOL_KEEPER market set-referred-discount-factor 10000000000000000000

cargo gmsol -w $GMSOL_KEEPER treasury set-gt-factor 51428600000000000000

cargo gmsol -w $GMSOL_KEEPER treasury set-buyback-factor 2000000000000000000

export TOKEN_MAP=$(cargo gmsol market create-token-map)
export ORACLE=$(cargo gmsol market init-oracle --seed $GMSOL_ORACLE_SEED --authority $CONFIG)
cargo gmsol -w $GMSOL_KEEPER market insert-token-configs $GMSOL_TOKENS --token-map $TOKEN_MAP --set-token-map
cargo gmsol -w $GMSOL_KEEPER market create-markets $GMSOL_MARKETS --enable

export SOL_WSOL_WSOL=$(solana find-program-derived-address $STORE_PROGRAM_ID string:market_token_mint pubkey:$STORE pubkey:$SOL pubkey:$WSOL pubkey:$WSOL)
export SOL_WSOL_USDG=$(solana find-program-derived-address $STORE_PROGRAM_ID string:market_token_mint pubkey:$STORE pubkey:$SOL pubkey:$WSOL pubkey:$USDG)
export BTC_BTC_USDG=$(solana find-program-derived-address $STORE_PROGRAM_ID string:market_token_mint pubkey:$STORE pubkey:$BTC pubkey:$BTC pubkey:$USDG)
export BTC_WSOL_USDG=$(solana find-program-derived-address $STORE_PROGRAM_ID string:market_token_mint pubkey:$STORE pubkey:$BTC pubkey:$WSOL pubkey:$USDG)

export BUFFER=$(cargo gmsol -w $GMSOL_KEEPER market push-to-buffer $GMSOL_MARKET_CONFIGS --init --market-token 11111111111111111111111111111112)
cargo gmsol -w $GMSOL_KEEPER market update-config $SOL_WSOL_WSOL --buffer $BUFFER

export BUFFER=$(cargo gmsol -w $GMSOL_KEEPER market push-to-buffer $GMSOL_MARKET_CONFIGS --init --market-token 11111111111111111111111111111113)
cargo gmsol -w $GMSOL_KEEPER market update-config $SOL_WSOL_USDG --buffer $BUFFER

export BUFFER=$(cargo gmsol -w $GMSOL_KEEPER market push-to-buffer $GMSOL_MARKET_CONFIGS --init --market-token 11111111111111111111111111111114)
cargo gmsol -w $GMSOL_KEEPER market update-config $BTC_BTC_USDG --buffer $BUFFER

export BUFFER=$(cargo gmsol -w $GMSOL_KEEPER market push-to-buffer $GMSOL_MARKET_CONFIGS --init --market-token 11111111111111111111111111111115)
cargo gmsol -w $GMSOL_KEEPER market update-config $BTC_WSOL_USDG --buffer $BUFFER

cargo gmsol -w $GMSOL_KEEPER market toggle-gt-minting $SOL_WSOL_WSOL --enable
cargo gmsol -w $GMSOL_KEEPER market toggle-gt-minting $SOL_WSOL_USDG --enable
cargo gmsol -w $GMSOL_KEEPER market toggle-gt-minting $BTC_BTC_USDG --enable
cargo gmsol -w $GMSOL_KEEPER market toggle-gt-minting $BTC_WSOL_USDG --enable

export COMMON_ALT=$(cargo gmsol alt extend --init common $ORACLE)
export MARKET_ALT=$(cargo gmsol alt extend --init market)

cargo gmsol timelock init-executor ADMIN
export ADMIN_EXECUTOR_WALLET=$(cargo gmsol timelock executor-wallet ADMIN)
cargo gmsol admin transfer-store-authority --new-authority $ADMIN_EXECUTOR_WALLET --confirm
cargo gmsol timelock init-config --initial-delay 300

echo "STORE: $STORE"
echo "ADMIN_EXECUTOR_WALLET: $ADMIN_EXECUTOR_WALLET"
echo "CONFIG: $CONFIG"
echo "RECEIVER: $RECEIVER"
echo "TREASURY: $TREASURY"
echo "ORACLE: $ORACLE"
echo "USDG: $USDG"
echo "BTC: $BTC"
echo "COMMON_ALT: $COMMON_ALT"
echo "MARKET_ALT: $MARKET_ALT"
