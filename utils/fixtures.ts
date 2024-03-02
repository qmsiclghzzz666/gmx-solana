import * as anchor from "@coral-xyz/anchor";

import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
chai.use(chaiAsPromised);

import { initializeRoleStore, roleStore } from "./role";

export const expect = chai.expect;

const provider = anchor.AnchorProvider.env();

// Users.
const user0 = anchor.web3.Keypair.generate();
const signer0 = anchor.web3.Keypair.generate();

export const getProvider = () => provider;

export const getPrograms = () => {
    return {
        roleStore,
    }
};

export const getUsers = () => {
    return {
        user0,
        signer0,
    }
};

export const mochaGlobalSetup = async () => {
    console.log("[Setting up everything...]");
    anchor.setProvider(provider);
    await initializeRoleStore(provider, signer0.publicKey);
    console.log("[Done.]");
};
