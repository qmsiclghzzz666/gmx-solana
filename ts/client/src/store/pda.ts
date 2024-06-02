import { PublicKey } from "@solana/web3.js";
import { DATA_STORE_ID } from "../program";
import { utils } from "@coral-xyz/anchor";

const encodeUtf8 = utils.bytes.utf8.encode;

export const POSITION_SEED = encodeUtf8("position");
export const ORDER_SEED = encodeUtf8("order");
export const CONFIG_SEED = utils.bytes.utf8.encode("config");

export const findRolesPDA = (store: PublicKey, authority: PublicKey) => PublicKey.findProgramAddressSync([
    encodeUtf8("roles"),
    store.toBytes(),
    authority.toBytes(),
], DATA_STORE_ID);

export const findTokenConfigMapPDA = (store: PublicKey) => PublicKey.findProgramAddressSync([
    encodeUtf8("token_config_map"),
    store.toBytes(),
], DATA_STORE_ID);

export const findMarketPDA = (store: PublicKey, token: PublicKey) => PublicKey.findProgramAddressSync([
    encodeUtf8("market"),
    store.toBytes(),
    token.toBytes(),
], DATA_STORE_ID);

export const findMarketVaultPDA = (store: PublicKey, tokenMint: PublicKey, marketTokenMint?: PublicKey) => PublicKey.findProgramAddressSync([
    encodeUtf8("market_vault"),
    store.toBytes(),
    tokenMint.toBytes(),
    marketTokenMint?.toBytes() ?? new Uint8Array(),
], DATA_STORE_ID);

export const findDepositPDA = (store: PublicKey, user: PublicKey, nonce: Uint8Array) => PublicKey.findProgramAddressSync([
    encodeUtf8("deposit"),
    store.toBytes(),
    user.toBytes(),
    nonce,
], DATA_STORE_ID);

export const findWithdrawalPDA = (store: PublicKey, user: PublicKey, nonce: Uint8Array) => PublicKey.findProgramAddressSync([
    encodeUtf8("withdrawal"),
    store.toBytes(),
    user.toBytes(),
    nonce,
], DATA_STORE_ID);

export const findPositionPDAWithKind = (store: PublicKey, user: PublicKey, marketToken: PublicKey, collateralToken: PublicKey, kind: number) => PublicKey.findProgramAddressSync([
    POSITION_SEED,
    store.toBytes(),
    user.toBytes(),
    marketToken.toBytes(),
    collateralToken.toBytes(),
    new Uint8Array([kind]),
], DATA_STORE_ID);

export const findPositionPDA = (store: PublicKey, user: PublicKey, marketToken: PublicKey, collateralToken: PublicKey, isLong: boolean) => findPositionPDAWithKind(
    store,
    user,
    marketToken,
    collateralToken,
    isLong ? 1 : 2
);

export const findOrderPDA = (store: PublicKey, user: PublicKey, nonce: Uint8Array) => PublicKey.findProgramAddressSync([
    ORDER_SEED,
    store.toBytes(),
    user.toBytes(),
    nonce,
], DATA_STORE_ID);


export const findConfigPDA = (store: PublicKey) => PublicKey.findProgramAddressSync([
    CONFIG_SEED,
    store.toBytes(),
], DATA_STORE_ID);
