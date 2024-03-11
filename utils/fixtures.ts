import * as anchor from "@coral-xyz/anchor";

import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
chai.use(chaiAsPromised);

import { createRoleStorePDA, initializeRoleStore, roleStore } from "./role";
import { createDataStorePDA, dataStore, initializeDataStore } from "./data";
import { createOraclePDA, initializeOracle } from "./oracle";
import { EventManager } from "./event";

export const expect = chai.expect;

const provider = anchor.AnchorProvider.env();

// Users.
const user0 = anchor.web3.Keypair.generate();
const signer0 = anchor.web3.Keypair.generate();

// Keys.
const roleStoreKey = "role_store_0";
const dataStoreKey = "data_store_0";
const oracleKey = "oracle_0";

// Addresses.
const [roleStoreAddress] = createRoleStorePDA(roleStoreKey);
const [dataStoreAddress] = createDataStorePDA(roleStoreAddress, dataStoreKey);
const [oracleAddress] = createOraclePDA(dataStoreAddress, oracleKey);

export const getProvider = () => provider;

export const getPrograms = () => {
    return {
        roleStore,
        dataStore,
    }
};

export const getUsers = () => {
    return {
        user0,
        signer0,
    }
};

export const getKeys = () => {
    return {
        roleStoreKey,
        dataStoreKey,
        oracleKey,
    }
};

export const getAddresses = () => {
    return {
        roleStoreAddress,
        dataStoreAddress,
        oracleAddress,
    }
}

const SHOW_EVENT = process.env.SHOW_EVENT;
const callback = SHOW_EVENT ? (eventName, event) => {
    console.debug(`<Event: ${eventName}>`, event);
} : (eventName, event) => { };

const eventManager = new EventManager(callback);

const initializeUser = async (provider: anchor.AnchorProvider, user: anchor.web3.Keypair, airdrop: number) => {
    // const tx = await provider.connection.requestAirdrop(user.publicKey, anchor.web3.LAMPORTS_PER_SOL * airdrop);
    // console.log(`Airdropped ${airdrop} SOL to the user ${user.publicKey} in tx ${tx}`);
    const balance = await provider.connection.getBalance(provider.wallet.publicKey);
    console.log(`The balance of ${provider.wallet.publicKey} is ${balance / anchor.web3.LAMPORTS_PER_SOL}`);
    const tx = new anchor.web3.Transaction().add(anchor.web3.SystemProgram.transfer({
        toPubkey: user.publicKey,
        fromPubkey: provider.wallet.publicKey,
        lamports: anchor.web3.LAMPORTS_PER_SOL * airdrop,
    }));
    tx.feePayer = provider.wallet.publicKey;
    try {
        const hash = await provider.sendAndConfirm(tx, [], { commitment: "confirmed" });
        console.log(`Transferred ${airdrop} SOL to ${user.publicKey} from the wallet in tx ${hash}`);
        const balance = await provider.connection.getBalance(user.publicKey);
        console.log(`Now it has ${balance} lamports`);
    } catch (error) {
        console.error("Failed to transfer:", error);
    }
};

const deinitializeUser = async (provider: anchor.AnchorProvider, user: anchor.web3.Keypair) => {
    const balance = await provider.connection.getBalance(user.publicKey);
    console.log(`The balance of ${user.publicKey} is ${balance / anchor.web3.LAMPORTS_PER_SOL}`);
    const tx = new anchor.web3.Transaction().add(anchor.web3.SystemProgram.transfer({
        fromPubkey: user.publicKey,
        toPubkey: provider.wallet.publicKey,
        lamports: balance,
    }));
    tx.feePayer = provider.wallet.publicKey;
    try {
        const hash = await provider.sendAndConfirm(tx, [user]);
        console.log(`Transferred back all balance of ${user.publicKey} to the wallet in tx ${hash}`);
    } catch (error) {
        console.error("Failed to transfer back all balance:", error);
    }
};

export const mochaGlobalSetup = async () => {
    console.log("[Setting up everything...]");
    anchor.setProvider(provider);
    await initializeUser(provider, signer0, 1.5);
    await initializeRoleStore(provider, roleStoreKey, signer0.publicKey);
    await initializeDataStore(eventManager, signer0, roleStoreKey, dataStoreKey);
    await initializeOracle(signer0, dataStoreAddress, oracleKey);
    console.log("[Done.]");
};

export const mochaGlobalTeardown = async () => {
    console.log("[Cleanup...]");
    eventManager.unsubscribeAll();
    await deinitializeUser(provider, signer0);
    console.log("[Done.]");
};
