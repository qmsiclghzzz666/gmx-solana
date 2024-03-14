import { PublicKey, Keypair } from '@solana/web3.js';

import { expect, getKeys, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { createControllerPDA, createRoleAdminPDA, createRoleStorePDA } from "../../utils/role";
import { createAddressPDA, createDataStorePDA, createTokenConfigPDA } from "../../utils/data";
import { AnchorError } from '@coral-xyz/anchor';

describe("data store: TokenConfig", () => {
    const provider = getProvider();
    const { dataStore } = getPrograms();
    const { signer0 } = getUsers();
    const { roleStoreKey, dataStoreKey } = getKeys();
    const [roleStorePDA] = createRoleStorePDA(roleStoreKey);
    const [dataStorePDA] = createDataStorePDA(roleStorePDA, dataStoreKey);

    const key = Keypair.generate().publicKey;
    const fooTokenConfigKey = `FOO:${key}`;
    const [fooTokenConfigPDA] = createTokenConfigPDA(dataStorePDA, fooTokenConfigKey);
    const fooAddress = Keypair.generate().publicKey;

    before("init token config", async () => {
        const [onlyController] = createControllerPDA(roleStorePDA, signer0.publicKey);
        await dataStore.methods.initializeTokenConfig(fooTokenConfigKey, fooAddress, 60, 18, 2).accounts({
            authority: signer0.publicKey,
            store: dataStorePDA,
            onlyController,
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
        const [otherMembership] = createRoleAdminPDA(roleStorePDA, provider.wallet.publicKey);
        await expect(dataStore.methods.updateTokenConfig(fooTokenConfigKey, fooAddress, 8, 4).accounts({
            authority: provider.wallet.publicKey,
            store: dataStorePDA,
            onlyController: otherMembership,
            tokenConfig: fooTokenConfigPDA,
        }).rpc()).to.be.rejectedWith(AnchorError, "Permission denied");
    });

    it("can be updated by CONTROLLER", async () => {
        const fooAddress = Keypair.generate().publicKey;
        const [onlyController] = createControllerPDA(roleStorePDA, signer0.publicKey);
        await dataStore.methods.updateTokenConfig(fooTokenConfigKey, fooAddress, 8, 4).accounts({
            authority: signer0.publicKey,
            store: dataStorePDA,
            onlyController,
            tokenConfig: fooTokenConfigPDA,
        }).signers([signer0]).rpc();
        const saved = await dataStore.account.tokenConfig.fetch(fooTokenConfigPDA);
        expect(saved.priceFeed).to.eql(fooAddress);
        expect(saved.precision).to.equal(4);
        expect(saved.tokenDecimals).to.equal(8);
    });
});
