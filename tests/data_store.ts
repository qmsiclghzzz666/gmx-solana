import { PublicKey, Keypair } from '@solana/web3.js';

import { expect, getKeys, getPrograms, getProvider, getUsers } from "../utils/fixtures";
import { createControllerPDA, createRoleAdminPDA, createRoleStorePDA } from "../utils/role";
import { createAddressPDA, createDataStorePDA } from "../utils/data";
import { AnchorError } from '@coral-xyz/anchor';

describe("data store", () => {
    const provider = getProvider();
    const { dataStore } = getPrograms();
    const { signer0 } = getUsers();
    const { roleStoreKey, dataStoreKey } = getKeys();
    const [roleStorePDA] = createRoleStorePDA(roleStoreKey);
    const [dataStorePDA] = createDataStorePDA(roleStorePDA, dataStoreKey);

    const key = Keypair.generate().publicKey;
    const fooAddressKey = `FOO:${key}`;
    const [fooAddressPDA] = createAddressPDA(dataStorePDA, fooAddressKey);

    it("set and get address", async () => {
        const fooAddress = Keypair.generate().publicKey;
        const [onlyController] = createControllerPDA(roleStorePDA, signer0.publicKey);
        await dataStore.methods.setAddress(fooAddressKey, fooAddress).accounts({
            authority: signer0.publicKey,
            store: dataStorePDA,
            onlyController,
            address: fooAddressPDA,
        }).signers([signer0]).rpc();
        const saved = await dataStore.methods.getAddress(fooAddressKey).accounts({
            store: dataStorePDA,
            address: fooAddressPDA,
        }).view() as PublicKey;
        expect(saved).to.eql(fooAddress);
    });

    it("can only be set by CONTROLLER", async () => {
        const fooAddress = Keypair.generate().publicKey;
        const [otherMembership] = createRoleAdminPDA(roleStorePDA, provider.wallet.publicKey);
        await expect(dataStore.methods.setAddress(fooAddressKey, fooAddress).accounts({
            authority: provider.wallet.publicKey,
            store: dataStorePDA,
            onlyController: otherMembership,
            address: fooAddressPDA,
        }).rpc()).to.be.rejectedWith(AnchorError, "Permission denied");
    });

    it("can be set again", async () => {
        const fooAddress = Keypair.generate().publicKey;
        const [onlyController] = createControllerPDA(roleStorePDA, signer0.publicKey);
        await dataStore.methods.setAddress(fooAddressKey, fooAddress).accounts({
            authority: signer0.publicKey,
            store: dataStorePDA,
            onlyController,
            address: fooAddressPDA,
        }).signers([signer0]).rpc();
        const saved = await dataStore.methods.getAddress(fooAddressKey).accounts({
            store: dataStorePDA,
            address: fooAddressPDA,
        }).view() as PublicKey;
        expect(saved).to.eql(fooAddress);
    });
});
