import * as anchor from "@coral-xyz/anchor";

import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
chai.use(chaiAsPromised);

import { initializeRoleStore, roleStore } from "./role";
import { dataStore } from "./data";

export const expect = chai.expect;

const provider = anchor.AnchorProvider.env();

// Users.
const user0 = anchor.web3.Keypair.generate();
const signer0 = anchor.web3.Keypair.generate();

const roleStoreKey = anchor.web3.Keypair.generate().publicKey.toBase58();

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
    }
};

const initializeUser = async (provider: anchor.AnchorProvider, user: anchor.web3.Keypair, airdrop: number) => {
    const tx = await provider.connection.requestAirdrop(user.publicKey, anchor.web3.LAMPORTS_PER_SOL * airdrop);
    console.log(`Airdropped ${airdrop} SOL to the user ${user.publicKey} in tx ${tx}`);
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
    await initializeUser(provider, signer0, 1);
    await initializeRoleStore(provider, roleStoreKey, signer0.publicKey);
    console.log("[Done.]");
};

export const mochaGlobalTeardown = async () => {
    console.log("[Cleanup...]");
    await deinitializeUser(provider, signer0);
    console.log("[Done.]");
};
