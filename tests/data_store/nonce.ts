import { Keypair, PublicKey } from '@solana/web3.js';

import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { createNoncePDA, createRolesPDA, createTokenConfigPDA } from "../../utils/data";
import { AnchorError } from '@coral-xyz/anchor';

describe("data store: Nonce", () => {
    const { dataStore } = getPrograms();
    const { signer0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let roles: PublicKey;
    let nonce: PublicKey;
    before(async () => {
        ({ dataStoreAddress } = await getAddresses());

        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
        [nonce] = createNoncePDA(dataStoreAddress);
    });

    it("inc nonce", async () => {
        const beforeNonce = await dataStore.methods.getNonceBytes().accounts({
            nonce,
        }).view();
        await dataStore.methods.incrementNonce().accounts({
            authority: signer0.publicKey,
            onlyController: roles,
            store: dataStoreAddress,
            nonce,
        }).signers([signer0]).rpc();
        const currentNonce = await dataStore.methods.getNonceBytes().accounts({
            nonce,
        }).view();
        expect(beforeNonce != currentNonce).true;
    });
});
