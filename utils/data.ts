import * as anchor from "@coral-xyz/anchor";
import { DataStore } from "../target/types/data_store";
import { keyToSeed } from "./seed";
import { createControllerPDA, createRoleStorePDA, roleStore } from "./role";
import { EventManager } from "./event";
import { PublicKey } from "@solana/web3.js";

export const dataStore = anchor.workspace.DataStore as anchor.Program<DataStore>;

export const DATA_STORE_SEED = anchor.utils.bytes.utf8.encode("data_store");
export const ADDRESS_SEED = anchor.utils.bytes.utf8.encode("address");
export const TOKEN_CONFIG_SEED = anchor.utils.bytes.utf8.encode("token_config");
export const MARKET_SEED = anchor.utils.bytes.utf8.encode("market");

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

export const createMarketPDA = (store: PublicKey, indexToken: PublicKey, longToken: PublicKey, shortToken: PublicKey) => PublicKey.findProgramAddressSync([
    MARKET_SEED,
    store.toBytes(),
    keyToSeed(`${indexToken}${longToken}${shortToken}`),
], dataStore.programId);

export const createKey = (prefix: string, key: string) => `${prefix}:${key}`;

export const createPriceFeedKey = key => createKey("PRICE_FEE", key);

export const BTC_TOKEN = anchor.translateAddress("3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh");
export const BTC_FEED = anchor.translateAddress("Cv4T27XbjVoKUYwP72NQQanvZeA7W4YF9L4EnYT9kx5o");
export const SOL_TOKEN = anchor.translateAddress("So11111111111111111111111111111111111111112");
export const SOL_FEED = anchor.translateAddress("CH31Xns5z3M1cTAbKW34jcxPPciazARpijcHj9rxtemt");

export const initializeDataStore = async (eventManager: EventManager, signer: anchor.web3.Keypair, roleStoreKey: string, dataStoreKey: string) => {
    const [roleStorePDA] = createRoleStorePDA(roleStoreKey);
    const [dataStorePDA] = createDataStorePDA(roleStorePDA, dataStoreKey);

    eventManager.subscribe(dataStore, "DataStoreInitEvent");
    eventManager.subscribe(dataStore, "TokenConfigChangeEvent");

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
        const key = BTC_TOKEN.toBase58();
        const [tokenConfigPDA] = createTokenConfigPDA(dataStorePDA, key);
        const tx = await dataStore.methods.initializeTokenConfig(key, BTC_FEED, 60, 8, 2).accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: createControllerPDA(roleStorePDA, signer.publicKey)[0],
            tokenConfig: tokenConfigPDA,
        }).signers([signer]).rpc();
        console.log(`Init a token config account ${tokenConfigPDA} for ${BTC_TOKEN} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to init the token config account", error);
    }

    // Insert SOL token config.
    try {
        const key = SOL_TOKEN.toBase58();
        const [tokenConfigPDA] = createTokenConfigPDA(dataStorePDA, key);
        const tx = await dataStore.methods.initializeTokenConfig(key, SOL_FEED, 60, 8, 4).accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: createControllerPDA(roleStorePDA, signer.publicKey)[0],
            tokenConfig: tokenConfigPDA,
        }).signers([signer]).rpc();
        console.log(`Init a token config account ${tokenConfigPDA} for ${SOL_TOKEN} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to init the token config account", error);
    }
};
