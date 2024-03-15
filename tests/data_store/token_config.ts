import { Keypair } from '@solana/web3.js';

import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { createRolesPDA, createTokenConfigPDA } from "../../utils/data";
import { AnchorError } from '@coral-xyz/anchor';

describe("data store: TokenConfig", () => {
    const provider = getProvider();
    const { dataStore } = getPrograms();
    const { signer0 } = getUsers();
    const { dataStoreAddress } = getAddresses();

    const [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);

    const key = Keypair.generate().publicKey;
    const fooTokenConfigKey = `FOO:${key}`;
    const [fooTokenConfigPDA] = createTokenConfigPDA(dataStoreAddress, fooTokenConfigKey);
    const fooAddress = Keypair.generate().publicKey;

    before("init token config", async () => {
        await dataStore.methods.initializeTokenConfig(fooTokenConfigKey, fooAddress, 60, 18, 2).accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyController: roles,
            tokenConfig: fooTokenConfigPDA,
        }).signers([signer0]).rpc({
            commitment: "confirmed",
        });
    });

    it("get token config", async () => {
        const saved = await dataStore.account.tokenConfig.fetch(fooTokenConfigPDA);
        expect(saved.priceFeed).to.eql(fooAddress);
        expect(saved.heartbeatDuration).to.equal(60);
        expect(saved.precision).to.equal(2);
        expect(saved.tokenDecimals).to.equal(18);
    });

    it("can only be updated by CONTROLLER", async () => {
        const fooAddress = Keypair.generate().publicKey;
        const [otherRoles] = createRolesPDA(dataStoreAddress, provider.wallet.publicKey);
        await expect(dataStore.methods.updateTokenConfig(fooTokenConfigKey, fooAddress, 8, 4).accounts({
            authority: provider.wallet.publicKey,
            store: dataStoreAddress,
            onlyController: otherRoles,
            tokenConfig: fooTokenConfigPDA,
        }).rpc()).to.be.rejectedWith(AnchorError, "Permission denied");
    });

    it("can be updated by CONTROLLER", async () => {
        const fooAddress = Keypair.generate().publicKey;
        await dataStore.methods.updateTokenConfig(fooTokenConfigKey, fooAddress, 8, 4).accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyController: roles,
            tokenConfig: fooTokenConfigPDA,
        }).signers([signer0]).rpc();
        const saved = await dataStore.account.tokenConfig.fetch(fooTokenConfigPDA);
        expect(saved.priceFeed).to.eql(fooAddress);
        expect(saved.precision).to.equal(4);
        expect(saved.tokenDecimals).to.equal(8);
    });
});
