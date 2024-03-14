import * as anchor from "@coral-xyz/anchor";
import { DataStore } from "../target/types/data_store";
import { keyToSeed } from "./seed";
import { createControllerPDA, createRoleStorePDA } from "./role";
import { EventManager } from "./event";
import { PublicKey } from "@solana/web3.js";
import { isDevNet } from "./endpoint";

export const dataStore = anchor.workspace.DataStore as anchor.Program<DataStore>;

const encodeUtf8 = anchor.utils.bytes.utf8.encode;

export const DATA_STORE_SEED = encodeUtf8("data_store");
export const ADDRESS_SEED = encodeUtf8("address");
export const TOKEN_CONFIG_SEED = encodeUtf8("token_config");
export const MARKET_SEED = encodeUtf8("market");
export const MARKET_SIGN_SEED = encodeUtf8("market_sign");
export const MARKET_TOKEN_MINT_SEED = encodeUtf8("market_token_mint");
export const MARKET_VAULT_SEED = encodeUtf8("market_vault");

export const createDataStorePDA = (role_store: anchor.web3.PublicKey, key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    DATA_STORE_SEED,
    role_store.toBytes(),
    keyToSeed(key),
], dataStore.programId);

export const createAddressPDA = (store: anchor.web3.PublicKey, key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    ADDRESS_SEED,
    store.toBytes(),
    keyToSeed(key),
], dataStore.programId);

export const createTokenConfigPDA = (store: anchor.web3.PublicKey, key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    TOKEN_CONFIG_SEED,
    store.toBytes(),
    keyToSeed(key),
], dataStore.programId);

export const createMarketPDA = (store: PublicKey, marketToken: PublicKey) => PublicKey.findProgramAddressSync([
    MARKET_SEED,
    store.toBytes(),
    keyToSeed(marketToken.toBase58()),
], dataStore.programId);

export const createMarketTokenMintPDA = (
    store: PublicKey,
    indexTokenMint: PublicKey,
    longTokenMint: PublicKey,
    shortTokenMint: PublicKey,
) => PublicKey.findProgramAddressSync([
    MARKET_TOKEN_MINT_SEED,
    store.toBytes(),
    indexTokenMint.toBytes(),
    longTokenMint.toBytes(),
    shortTokenMint.toBytes(),
], dataStore.programId);

export const createMarketVaultPDA = (store: PublicKey, tokenMint: PublicKey, marketTokenMint?: PublicKey) => PublicKey.findProgramAddressSync([
    MARKET_VAULT_SEED,
    store.toBytes(),
    tokenMint.toBytes(),
    marketTokenMint?.toBytes() ?? new Uint8Array(),
], dataStore.programId);

export const getMarketSignPDA = () => PublicKey.findProgramAddressSync([MARKET_SIGN_SEED], dataStore.programId);

export const BTC_TOKEN_MINT = anchor.translateAddress(isDevNet ? "Hb5pJ53KeUPCkUvaDZm7Y7WafEjuP1xjD4owaXksJ86R" : "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh");
export const BTC_FEED = anchor.translateAddress(isDevNet ? "6PxBx93S8x3tno1TsFZwT5VqP8drrRCbCXygEXYNkFJe" : "Cv4T27XbjVoKUYwP72NQQanvZeA7W4YF9L4EnYT9kx5o");
export const SOL_TOKEN_MINT = anchor.translateAddress("So11111111111111111111111111111111111111112");
export const SOL_FEED = anchor.translateAddress(isDevNet ? "99B2bTijsU6f1GCT73HmdR7HCFFjGMBcPZY6jZ96ynrR" : "CH31Xns5z3M1cTAbKW34jcxPPciazARpijcHj9rxtemt");

export const initializeDataStore = async (eventManager: EventManager, signer: anchor.web3.Keypair, roleStoreKey: string, dataStoreKey: string) => {
    const [roleStorePDA] = createRoleStorePDA(roleStoreKey);
    const [dataStorePDA] = createDataStorePDA(roleStorePDA, dataStoreKey);

    eventManager.subscribe(dataStore, "DataStoreInitEvent");
    eventManager.subscribe(dataStore, "TokenConfigChangeEvent");
    eventManager.subscribe(dataStore, "MarketChangeEvent");

    // Initialize a DataStore with the given key.
    try {
        const tx = await dataStore.methods.initialize(dataStoreKey).accounts({
            authority: signer.publicKey,
            roleStore: roleStorePDA,
            dataStore: dataStorePDA,
        }).signers([signer]).rpc();
        console.log(`Initialized a new data store account ${dataStorePDA.toBase58()} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to initialize a data store with the given key:", error);
    }

    // Insert BTC token config.
    try {
        const key = BTC_TOKEN_MINT.toBase58();
        const [tokenConfigPDA] = createTokenConfigPDA(dataStorePDA, key);
        const tx = await dataStore.methods.initializeTokenConfig(key, BTC_FEED, 60, 8, 2).accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: createControllerPDA(roleStorePDA, signer.publicKey)[0],
            tokenConfig: tokenConfigPDA,
        }).signers([signer]).rpc();
        console.log(`Init a token config account ${tokenConfigPDA} for ${BTC_TOKEN_MINT} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to init the token config account", error);
    }

    // Insert SOL token config.
    try {
        const key = SOL_TOKEN_MINT.toBase58();
        const [tokenConfigPDA] = createTokenConfigPDA(dataStorePDA, key);
        const tx = await dataStore.methods.initializeTokenConfig(key, SOL_FEED, 60, 9, 4).accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: createControllerPDA(roleStorePDA, signer.publicKey)[0],
            tokenConfig: tokenConfigPDA,
        }).signers([signer]).rpc();
        console.log(`Init a token config account ${tokenConfigPDA} for ${SOL_TOKEN_MINT} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to init the token config account", error);
    }
};
