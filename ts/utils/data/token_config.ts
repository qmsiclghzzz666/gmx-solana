import { PublicKey, Signer } from "@solana/web3.js";
import { dataStore } from "./program";
import { createRolesPDA } from ".";
import { utils } from "@coral-xyz/anchor";

// Token Config map seed.
export const TOKEN_CONFIG_MAP_SEED = utils.bytes.utf8.encode("token_config_map");

export const createTokenConfigMapPDA = (store: PublicKey) => PublicKey.findProgramAddressSync([
    TOKEN_CONFIG_MAP_SEED,
    store.toBytes(),
], dataStore.programId);

export const initializeTokenConfigMap = async (authority: Signer, store: PublicKey, len: number) => {
    const [map] = createTokenConfigMapPDA(store);
    await dataStore.methods.initializeTokenConfigMap(len).accounts({
        authority: authority.publicKey,
        store,
        onlyController: createRolesPDA(store, authority.publicKey)[0],
    }).signers([authority]).rpc();
    return map;
};

export const insertTokenConfig = async (
    authority: Signer,
    store: PublicKey,
    token: PublicKey,
    price_feed: PublicKey,
    heartbeat_duration: number,
    precision: number,
) => {
    await dataStore.methods.insertTokenConfig(price_feed, heartbeat_duration, precision).accounts({
        authority: authority.publicKey,
        store,
        onlyController: createRolesPDA(store, authority.publicKey)[0],
        token,
    }).signers([authority]).rpc();
};

export const toggleTokenConfig = async (
    authority: Signer,
    store: PublicKey,
    token: PublicKey,
    enable: boolean,
) => {
    await dataStore.methods.toggleTokenConfig(token, enable).accounts({
        authority: authority.publicKey,
        store,
        onlyController: createRolesPDA(store, authority.publicKey)[0],
    }).signers([authority]).rpc();
};

export interface TokenConfig {
    enabled: boolean,
    priceFeed: PublicKey,
    heartbeatDuration: number,
    tokenDecimals: number,
    precision: number,
}

export const getTokenConfig = async (store: PublicKey, token: PublicKey) => {
    const config: TokenConfig = await dataStore.methods.getTokenConfig(store, token).accounts({
        map: createTokenConfigMapPDA(store)[0],
    }).view();
    return config;
}

export const extendTokenConfigMap = async (authority: Signer, store: PublicKey, extendLen: number) => {
    await dataStore.methods.extendTokenConfigMap(extendLen).accounts({
        authority: authority.publicKey,
        store,
        onlyController: createRolesPDA(store, authority.publicKey)[0],
    }).signers([authority]).rpc();
};
