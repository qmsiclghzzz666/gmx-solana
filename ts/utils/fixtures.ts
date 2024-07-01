// Must be imported first to make sure the anchor is initialized with the given provider.
import { isDevNet, provider } from "./endpoint";

import * as anchor from "@coral-xyz/anchor";
import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
chai.use(chaiAsPromised);

import { setInitialized, waitForSetup } from "./setup";
import { EventManager } from "./event";
import { createDataStorePDA, createMarketTokenMintPDA, createMarketVault, createOraclePDA, dataStore, initializeDataStore } from "./data";
import { initializeMarkets, exchange, invokeInitializeController } from "./exchange";

import { IDL as chainlinkIDL } from "../../external-programs/chainlink-store";
import { BTC_TOKEN_MINT, SOL_TOKEN_MINT, createSignedToken } from "./token";
import { PublicKey, Transaction } from "@solana/web3.js";
import { createAssociatedTokenAccount, createWrappedNativeAccount } from "@solana/spl-token";
import { CHAINLINK_ID } from "./external";

export const expect = chai.expect;

// Get anchor provider.
export const getProvider = () => provider;

// Users.
const user0 = anchor.web3.Keypair.generate();
const signer0 = anchor.web3.Keypair.generate();

export const getUsers = () => {
    return {
        user0,
        signer0,
    }
};

// Keys.
const randomeKey = anchor.web3.Keypair.generate().publicKey.toBase58();
const dataStoreKey = isDevNet ? randomeKey.slice(0, 10) : "data_store_0";
const oracleIndex = 255;

export const getKeys = () => {
    return {
        dataStoreKey,
    }
};

// Addresses.
const [dataStoreAddress] = createDataStorePDA(dataStoreKey);
const [oracleAddress] = createOraclePDA(dataStoreAddress, oracleIndex);

let user0FakeTokenAccount: PublicKey;
let user0UsdGTokenAccount: PublicKey;
let user0WsolTokenAccount: PublicKey;
let user0FakeFakeUsdGTokenAccount: PublicKey;
let user0WsolWsolUsdGTokenAccount: PublicKey;
let fakeTokenMint: PublicKey;
let usdGTokenMint: PublicKey;
let fakeTokenVault: PublicKey;
let usdGVault: PublicKey;
let wsolVault: PublicKey;

export const getAddresses = async () => {
    await waitForSetup();
    return {
        dataStoreAddress,
        oracleAddress,
        user0FakeTokenAccount,
        user0UsdGTokenAccount,
        user0FakeFakeUsdGTokenAccount,
        user0WsolTokenAccount,
        user0WsolWsolUsdGTokenAccount,
        fakeTokenMint,
        usdGTokenMint,
        fakeTokenVault,
        usdGVault,
        wsolVault,
    }
}

let markets: Awaited<ReturnType<typeof initializeMarkets>>;
export const getMarkets = async () => {
    await waitForSetup();
    return markets;
}

export const getPrograms = () => {
    return {
        dataStore,
        exchange,
    }
};

export const getTokenMints = () => {
    return {
        BTC_TOKEN_MINT,
        SOL_TOKEN_MINT,
    }
};

const SHOW_EVENT = process.env.SHOW_EVENT;
const callback = SHOW_EVENT ? (eventName, event) => {
    console.debug(`<Event: ${eventName}>`, event);
} : (eventName, event) => { };

const eventManager = new EventManager(callback);

const initializeUsers = async (provider: anchor.AnchorProvider, users: anchor.web3.Keypair[], airdrop: number) => {
    console.log("Using", provider.connection.rpcEndpoint);
    // const tx = await provider.connection.requestAirdrop(user.publicKey, anchor.web3.LAMPORTS_PER_SOL * airdrop);
    // console.log(`Airdropped ${airdrop} SOL to the user ${user.publicKey} in tx ${tx}`);
    const balance = await provider.connection.getBalance(provider.wallet.publicKey);
    console.log(`The balance of ${provider.wallet.publicKey} is ${balance / anchor.web3.LAMPORTS_PER_SOL}`);
    const tx = new Transaction();
    for (let index = 0; index < users.length; index++) {
        const user = users[index];
        tx.add(anchor.web3.SystemProgram.transfer({
            toPubkey: user.publicKey,
            fromPubkey: provider.wallet.publicKey,
            lamports: anchor.web3.LAMPORTS_PER_SOL * airdrop,
        }));
    }
    tx.feePayer = provider.wallet.publicKey;
    try {
        const hash = await provider.sendAndConfirm(tx, [], { commitment: "confirmed" });
        for (let index = 0; index < users.length; index++) {
            const user = users[index];
            console.log(`Transferred ${airdrop} SOL to ${user.publicKey} from the wallet in tx ${hash}`);
            const balance = await provider.connection.getBalance(user.publicKey);
            console.log(`Now it has ${balance} lamports`);
        }
    } catch (error) {
        console.error("Failed to transfer:", error);
    }
};

const deinitializeUsers = async (provider: anchor.AnchorProvider, users: anchor.web3.Keypair[]) => {
    const tx = new Transaction();
    for (let index = 0; index < users.length; index++) {
        const user = users[index];
        const balance = await provider.connection.getBalance(user.publicKey);
        console.log(`The balance of ${user.publicKey} is ${balance / anchor.web3.LAMPORTS_PER_SOL}`);
        tx.add(anchor.web3.SystemProgram.transfer({
            fromPubkey: user.publicKey,
            toPubkey: provider.wallet.publicKey,
            lamports: balance,
        }));
    }

    tx.feePayer = provider.wallet.publicKey;
    try {
        const hash = await provider.sendAndConfirm(tx, users);
        console.log(`Transferred back all balance of the users to the wallet in tx ${hash}`);
    } catch (error) {
        console.error("Failed to transfer back all balance:", error);
    }
};

export const mochaGlobalSetup = async () => {
    try {
        console.log("[Setting up everything...]");
        anchor.setProvider(provider);
        await initializeUsers(provider, [signer0, user0], 2);

        // Init fakeToken and usdG.
        const fakeToken = await createSignedToken(signer0, 9);
        fakeTokenMint = fakeToken.mint;
        const usdG = await createSignedToken(signer0, 8);
        usdGTokenMint = usdG.mint;
        user0FakeTokenAccount = await fakeToken.createTokenAccount(user0.publicKey);
        user0UsdGTokenAccount = await usdG.createTokenAccount(user0.publicKey);
        user0WsolTokenAccount = await createWrappedNativeAccount(provider.connection, user0, user0.publicKey, 10_000_000);
        fakeToken.mintTo(user0FakeTokenAccount, 1_000 * 1_000_000_000);
        usdG.mintTo(user0UsdGTokenAccount, 1_000_000 * 100_000_000);

        await initializeDataStore(provider, eventManager, signer0, user0, dataStoreKey, oracleIndex, fakeTokenMint, usdGTokenMint);
        await invokeInitializeController(exchange, { payer: signer0, store: dataStoreAddress });

        // fakeTokenVault = await createMarketVault(provider, signer0, dataStoreAddress, fakeTokenMint);
        // usdGVault = await createMarketVault(provider, signer0, dataStoreAddress, usdGTokenMint);
        // wsolVault = await createMarketVault(provider, signer0, dataStoreAddress, SOL_TOKEN_MINT);

        markets = await initializeMarkets(signer0, dataStoreAddress, fakeTokenMint, usdGTokenMint);
        user0FakeFakeUsdGTokenAccount = await createAssociatedTokenAccount(provider.connection, user0, markets.GMFakeFakeUsdG, user0.publicKey);
        user0WsolWsolUsdGTokenAccount = await createAssociatedTokenAccount(provider.connection, user0, markets.GMWsolWsolUsdG, user0.publicKey);

        eventManager.subscribe(exchange, "DepositCreatedEvent");

        console.log("[Done.]");
    } catch (error) {
        console.log("[Failed.]", error);
    } finally {
        setInitialized();
    }
};

export const mochaGlobalTeardown = async () => {
    console.log("[Cleanup...]");
    eventManager.unsubscribeAll();
    await deinitializeUsers(provider, [signer0, user0]);
    console.log("[Done.]");
};
