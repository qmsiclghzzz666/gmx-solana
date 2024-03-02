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
    }
};

export const mochaGlobalSetup = async () => {
    console.log("[Setting up everything...]");
    anchor.setProvider(provider);
    await initializeRoleStore(provider, provider.wallet.publicKey);
    console.log("[Done.]");
};
