import { Keypair, PublicKey } from '@solana/web3.js';

import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { createRolesPDA, createTokenConfigPDA, extendTokenConfigMap, getTokenConfig, insertTokenConfig, toggleTokenConfig } from "../../utils/data";
import { AnchorError } from '@coral-xyz/anchor';
import { createMint } from '@solana/spl-token';
import { BTC_FEED, SOL_FEED } from '../../utils/token';

describe("data store: TokenConfig", () => {
    const provider = getProvider();
    const { dataStore } = getPrograms();
    const { signer0 } = getUsers();

    const key = Keypair.generate().publicKey;
    const fooTokenConfigKey = `FOO:${key}`;
    const fooAddress = Keypair.generate().publicKey;

    let dataStoreAddress: PublicKey;
    let roles: PublicKey;
    let fooTokenConfigPDA: PublicKey;
    let fakeTokenMint: PublicKey;
    before("init token config", async () => {
        ({ dataStoreAddress, fakeTokenMint } = await getAddresses());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
        [fooTokenConfigPDA] = createTokenConfigPDA(dataStoreAddress, fooTokenConfigKey);

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

    it("test token config map", async () => {
        const newToken = await createMint(provider.connection, signer0, signer0.publicKey, signer0.publicKey, 10);

        // Config not exists yet.
        {
            const config = await getTokenConfig(dataStoreAddress, newToken);
            expect(config).null;
        }

        // Shouldn't have enough space for inserting a new token config.
        await expect(insertTokenConfig(signer0, dataStoreAddress, newToken, BTC_FEED, 60, 3)).to.be.rejectedWith(AnchorError, "AccountDidNotSerialize");

        // Extend the map.
        await extendTokenConfigMap(signer0, dataStoreAddress, 1);

        // We should be able to insert now.
        {
            await insertTokenConfig(signer0, dataStoreAddress, newToken, BTC_FEED, 60, 3);
            const config = await getTokenConfig(dataStoreAddress, newToken);
            expect(config.enabled).true;
            expect(config.priceFeed).eqls(BTC_FEED);
            expect(config.heartbeatDuration).equals(60);
            expect(config.precision).equals(3);
        }

        // Update the config by inserting again.
        {
            await insertTokenConfig(signer0, dataStoreAddress, newToken, SOL_FEED, 42, 5);
            const config = await getTokenConfig(dataStoreAddress, newToken);
            expect(config.enabled).true;
            expect(config.priceFeed).eqls(SOL_FEED);
            expect(config.heartbeatDuration).equals(42);
            expect(config.precision).equals(5);
        }

        // We can disable the config temporarily.
        {
            await toggleTokenConfig(signer0, dataStoreAddress, newToken, false);
            const config = await getTokenConfig(dataStoreAddress, newToken);
            expect(config.enabled).false;
        }

        // And we can enable the config again.
        {
            await toggleTokenConfig(signer0, dataStoreAddress, newToken, true);
            const config = await getTokenConfig(dataStoreAddress, newToken);
            expect(config.enabled).true;
        }
    });
});
