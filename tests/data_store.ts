import { PublicKey, Keypair } from '@solana/web3.js';

import { expect, getPrograms, getProvider, getUsers } from "../utils/fixtures";
import { createControllerPDA, createMembershipPDA, roleStore } from "../utils/role";
import { createAddressPDA } from "../utils/data";

describe("data store", () => {
    const provider = getProvider();
    const { dataStore } = getPrograms();
    const { user0 } = getUsers();

    const key = Keypair.generate().publicKey;
    const fooAddressKey = `PRICE_FEED:${key}`;
    const fooAddress = Keypair.generate().publicKey;
    const [fooAddressPDA] = createAddressPDA(fooAddressKey);

    it("set and get address", async () => {
        const [onlyController] = createControllerPDA(provider.wallet.publicKey);
        await dataStore.methods.setAddress(fooAddressKey, fooAddress).accounts({
            authority: provider.wallet.publicKey,
            onlyController,
            address: fooAddressPDA,
        }).signers([]).rpc();
        const saved = await dataStore.methods.getAddress(fooAddressKey).accounts({
            address: fooAddressPDA,
        }).view() as PublicKey;
        expect(saved).to.eql(fooAddress);
    });

    it("only controller can set address", async () => {
        const [otherMembership] = createMembershipPDA("OTHER", user0.publicKey);
        await expect(dataStore.methods.setAddress(fooAddressKey, fooAddress).accounts({
            authority: user0.publicKey,
            onlyController: otherMembership,
            address: fooAddressPDA,
        }).signers([user0]).rpc()).to.be.rejected;
    });
});
