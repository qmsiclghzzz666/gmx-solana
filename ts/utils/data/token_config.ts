import { PublicKey, Signer } from "@solana/web3.js";
import { dataStore } from "./program";
import { createRolesPDA } from ".";
import { utils } from "@coral-xyz/anchor";
import { DataStoreProgram, PriceProvider, makeInvoke, toBN } from "gmsol";

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
    }).signers([authority]).rpc();
    return map;
};

const hexStringToPublicKey = (hex: string) => {
    const decoded = utils.bytes.hex.decode(hex);
    return new PublicKey(decoded);
};

export interface FeedsOptions {
    pythFeedId?: string,
    chainlinkFeed?: PublicKey,
    pythDevFeed?: PublicKey,
    expectedProvider?: PriceProvider,
}

export const insertTokenConfig = async (
    authority: Signer,
    store: PublicKey,
    token: PublicKey,
    heartbeatDuration: number,
    precision: number,
    {
        pythFeedId,
        chainlinkFeed,
        pythDevFeed,
        expectedProvider,
    }: FeedsOptions,
) => {
    await dataStore.methods.insertTokenConfig({
        heartbeatDuration,
        precision,
        feeds: [
            pythFeedId ? hexStringToPublicKey(pythFeedId) : PublicKey.default,
            chainlinkFeed ?? PublicKey.default,
            pythDevFeed ?? PublicKey.default,
        ],
        expectedProvider,
    }, true).accounts({
        authority: authority.publicKey,
        store,
        token,
    }).signers([authority]).rpc();
};

export const insertSyntheticTokenConfig = async (
    authority: Signer,
    store: PublicKey,
    token: PublicKey,
    decimals: number,
    heartbeatDuration: number,
    precision: number,
    {
        pythFeedId,
        chainlinkFeed,
        pythDevFeed,
        expectedProvider,
    }: FeedsOptions,
) => {
    await dataStore.methods.insertSyntheticTokenConfig(token, decimals, {
        heartbeatDuration,
        precision,
        feeds: [
            pythFeedId ? hexStringToPublicKey(pythFeedId) : PublicKey.default,
            chainlinkFeed ?? PublicKey.default,
            pythDevFeed ?? PublicKey.default,
        ],
        expectedProvider,
    }, true).accounts({
        authority: authority.publicKey,
        store,
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
    }).signers([authority]).rpc();
};

export const setExpectedProvider = async (
    authority: Signer,
    store: PublicKey,
    token: PublicKey,
    provider: PriceProvider,
) => {
    await dataStore.methods.setExpectedProvider(token, provider).accounts({
        authority: authority.publicKey,
        store,
    }).signers([authority]).rpc();
};

export interface TokenConfig {
    enabled: boolean,
    heartbeatDuration: number,
    tokenDecimals: number,
    precision: number,
    feeds: PublicKey[],
    expectedProvider: number,
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
    }).signers([authority]).rpc();
};

export const makeInsertTokenConfigAmountInstruction = async (
    program: DataStoreProgram,
    { authority, store, token, key, amount }: {
        authority: PublicKey,
        store: PublicKey,
        token: PublicKey,
        key: string,
        amount: number | bigint,
    }
) => {
    return await program.methods.insertTokenConfigAmount(token, key, toBN(amount)).accounts({
        authority,
        store,
    }).instruction();
};

export const invokeInsertTokenConfigAmount = makeInvoke(makeInsertTokenConfigAmountInstruction, ["authority"]);
