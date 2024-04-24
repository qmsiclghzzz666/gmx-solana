import { PublicKey } from "@solana/web3.js";
import { DATA_STORE_ID } from "../program";
import { utils } from "@coral-xyz/anchor";
import { keyToSeed } from "../utils/seed";

const encodeUtf8 = utils.bytes.utf8.encode;

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
    keyToSeed(token.toBase58()),
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
