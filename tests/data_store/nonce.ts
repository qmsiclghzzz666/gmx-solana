import { Keypair } from '@solana/web3.js';

import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { createNoncePDA, createRolesPDA, createTokenConfigPDA } from "../../utils/data";
import { AnchorError } from '@coral-xyz/anchor';

describe("data store: Nonce", () => {
    const provider = getProvider();
    const { dataStore } = getPrograms();
    const { signer0 } = getUsers();
    const { dataStoreAddress } = getAddresses();

    const [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
    const [nonce] = createNoncePDA(dataStoreAddress);

    it("inc nonce", async () => {
        await dataStore.methods.incrementNonce().accounts({
            authority: signer0.publicKey,
            onlyController: roles,
            payer: provider.publicKey,
            store: dataStoreAddress,
            nonce,
        }).signers([signer0]).rpc();
        const currentNonce = await dataStore.account.nonce.fetch(nonce);
        console.log(currentNonce);
    });
});
