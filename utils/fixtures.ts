// Must be imported first to make sure the anchor is initialized with the given provider.
import { isDevNet, provider } from "./endpoint";

import * as anchor from "@coral-xyz/anchor";
import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
chai.use(chaiAsPromised);

import { setInitialized, waitForSetup } from "./setup";
import { EventManager } from "./event";
import { createDataStorePDA, createMarketTokenMintPDA, createMarketVault, createOraclePDA, dataStore, initializeDataStore } from "./data";
import { initializeMarkets, exchange } from "./exchange";
import { oracle } from "./oracle";

import { IDL as chainlinkIDL } from "../external-programs/chainlink-store";
import { BTC_TOKEN_MINT, SOL_TOKEN_MINT, createSignedToken } from "./token";
import { PublicKey } from "@solana/web3.js";
import { createAssociatedTokenAccount } from "@solana/spl-token";

export const expect = chai.expect;

// Get anchor provider.
export const getProvider = () => provider;

// External Program IDs.
const chainlinkID = "HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny";

export const getExternalPrograms = () => {
    return {
        chainlink: new anchor.Program(chainlinkIDL, chainlinkID),
    }
};

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
const dataStoreKey = isDevNet ? randomeKey : "data_store_0";
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
let user0FakeFakeUsdGTokenAccount: PublicKey;
let fakeTokenMint: PublicKey;
let usdGTokenMint: PublicKey;
let fakeTokenVault: PublicKey;
let usdGVault: PublicKey;

export const getAddresses = async () => {
    await waitForSetup();
    return {
        dataStoreAddress,
        oracleAddress,
        user0FakeTokenAccount,
        user0UsdGTokenAccount,
        user0FakeFakeUsdGTokenAccount,
        fakeTokenMint,
        usdGTokenMint,
        fakeTokenVault,
        usdGVault,
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
        oracle,
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

const initializeUser = async (provider: anchor.AnchorProvider, user: anchor.web3.Keypair, airdrop: number) => {
    console.log("Using", provider.connection.rpcEndpoint);
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
    await initializeUser(provider, user0, 1.5);

    // Init fakeToken and usdG.
    const fakeToken = await createSignedToken(signer0, 9);
    fakeTokenMint = fakeToken.mint;
    const usdG = await createSignedToken(signer0, 8);
    usdGTokenMint = usdG.mint;
    user0FakeTokenAccount = await fakeToken.createTokenAccount(user0.publicKey);
    user0UsdGTokenAccount = await usdG.createTokenAccount(user0.publicKey);
    fakeToken.mintTo(user0FakeTokenAccount, 1_000 * 1_000_000_000);
    usdG.mintTo(user0UsdGTokenAccount, 1_000 * 100_000_000);

    await initializeDataStore(provider, eventManager, signer0, dataStoreKey, oracleIndex, fakeTokenMint, usdGTokenMint);

    fakeTokenVault = await createMarketVault(provider, signer0, dataStoreAddress, fakeTokenMint);
    usdGVault = await createMarketVault(provider, signer0, dataStoreAddress, usdGTokenMint);

    markets = await initializeMarkets(signer0, dataStoreAddress, fakeTokenMint, usdGTokenMint);
    const [marketTokenMint] = createMarketTokenMintPDA(dataStoreAddress, fakeTokenMint, fakeTokenMint, usdGTokenMint);
    user0FakeFakeUsdGTokenAccount = await createAssociatedTokenAccount(provider.connection, user0, marketTokenMint, user0.publicKey);

    console.log("[Done.]");
    setInitialized();
};

export const mochaGlobalTeardown = async () => {
    console.log("[Cleanup...]");
    eventManager.unsubscribeAll();
    await deinitializeUser(provider, signer0);
    await deinitializeUser(provider, user0);
    console.log("[Done.]");
};
