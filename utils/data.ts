import * as anchor from "@coral-xyz/anchor";
import { DataStore } from "../target/types/data_store";
import { keyToSeed } from "./seed";
import { createControllerPDA, createRoleStorePDA } from "./role";

export const dataStore = anchor.workspace.DataStore as anchor.Program<DataStore>;

export const DATA_STORE_SEED = anchor.utils.bytes.utf8.encode("data_store");
export const ADDRESS_SEED = anchor.utils.bytes.utf8.encode("address");

export const createAddressPDA = (store: anchor.web3.PublicKey, key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    ADDRESS_SEED,
    store.toBytes(),
    keyToSeed(key),
], dataStore.programId);

export const createDataStorePDA = (role_store: anchor.web3.PublicKey, key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    DATA_STORE_SEED,
    role_store.toBytes(),
    keyToSeed(key),
], dataStore.programId);

export const createKey = (prefix: string, key: string) => `${prefix}:${key}`;

export const createPriceFeedKey = key => createKey("PRICE_FEE", key);

export const BTC_TOKEN = anchor.translateAddress("3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh");
export const BTC_FEED = anchor.translateAddress("Cv4T27XbjVoKUYwP72NQQanvZeA7W4YF9L4EnYT9kx5o");

export const initializeDataStore = async (signer: anchor.web3.Keypair, roleStoreKey: string, dataStoreKey: string) => {
    const [roleStorePDA] = createRoleStorePDA(roleStoreKey);
    const [dataStorePDA] = createDataStorePDA(roleStorePDA, dataStoreKey);

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

    // Insert BTC feed.
    try {
        const key = createPriceFeedKey(BTC_TOKEN);
        const [addressPDA] = createAddressPDA(dataStorePDA, key);
        const tx = await dataStore.methods.setAddress(key, BTC_FEED).accounts({
            authority: signer.publicKey,
            store: dataStorePDA,
            onlyController: createControllerPDA(roleStorePDA, signer.publicKey)[0],
            address: addressPDA,
        }).signers([signer]).rpc();
        console.log(`Set an address account ${addressPDA} for ${BTC_TOKEN} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to set address account", error);
    }
};
