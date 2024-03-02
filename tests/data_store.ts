import { PublicKey, Keypair } from '@solana/web3.js';

import { expect, getPrograms, getProvider, getUsers } from "../utils/fixtures";
import { createControllerPDA, createMembershipPDA, roleStore } from "../utils/role";
import { createAddressPDA } from "../utils/data";

describe("data store", () => {
    const { dataStore } = getPrograms();
    const { user0, signer0 } = getUsers();

    const key = Keypair.generate().publicKey;
    const fooAddressKey = `PRICE_FEED:${key}`;
    const [fooAddressPDA] = createAddressPDA(fooAddressKey);

    it("set and get address", async () => {
        const fooAddress = Keypair.generate().publicKey;
        const [onlyController] = createControllerPDA(signer0.publicKey);
        await dataStore.methods.setAddress(fooAddressKey, fooAddress).accounts({
            authority: signer0.publicKey,
            onlyController,
            address: fooAddressPDA,
        }).signers([signer0]).rpc();
        const saved = await dataStore.methods.getAddress(fooAddressKey).accounts({
            address: fooAddressPDA,
        }).view() as PublicKey;
        expect(saved).to.eql(fooAddress);
    });

    it("can only be set by controller", async () => {
        const fooAddress = Keypair.generate().publicKey;
        const [otherMembership] = createMembershipPDA("OTHER", user0.publicKey);
        await expect(dataStore.methods.setAddress(fooAddressKey, fooAddress).accounts({
            authority: user0.publicKey,
            onlyController: otherMembership,
            address: fooAddressPDA,
        }).signers([user0]).rpc()).to.be.rejected;
    });

    it("can be set again", async () => {
        const fooAddress = Keypair.generate().publicKey;
        const [onlyController] = createControllerPDA(signer0.publicKey);
        await dataStore.methods.setAddress(fooAddressKey, fooAddress).accounts({
            authority: signer0.publicKey,
            onlyController,
            address: fooAddressPDA,
        }).signers([signer0]).rpc();
        const saved = await dataStore.methods.getAddress(fooAddressKey).accounts({
            address: fooAddressPDA,
        }).view() as PublicKey;
        expect(saved).to.eql(fooAddress);
    });
});
