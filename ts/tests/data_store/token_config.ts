import { Keypair, PublicKey } from '@solana/web3.js';

import { expect, getAddresses, getProvider, getUsers } from "../../utils/fixtures";
import { createRolesPDA, extendTokenConfigMap, getTokenConfig, insertSyntheticTokenConfig, insertTokenConfig, toggleTokenConfig } from "../../utils/data";
import { AnchorError } from '@coral-xyz/anchor';
import { createMint } from '@solana/spl-token';
import { BTC_FEED, SOL_FEED } from '../../utils/token';

describe("data store: TokenConfig", () => {
    const provider = getProvider();
    const { signer0, user0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let roles: PublicKey;
    let fakeTokenMint: PublicKey;
    before("init token config", async () => {
        ({ dataStoreAddress, fakeTokenMint } = await getAddresses());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
    });

    it("can only be updated by CONTROLLER", async () => {
        const fooAddress = Keypair.generate().publicKey;
        await expect(insertTokenConfig(user0, dataStoreAddress, fakeTokenMint, fooAddress, 123, 10)).to.be.rejectedWith(AnchorError, "Permission denied");
    });

    it("test token config map", async () => {
        const newToken = await createMint(provider.connection, signer0, signer0.publicKey, signer0.publicKey, 10);

        // Config not exists yet.
        {
            const config = await getTokenConfig(dataStoreAddress, newToken);
            expect(config).null;
        }

        // We should be able to insert.
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

    it("insert fake token", async () => {
        const newFakeToken = PublicKey.unique();
        // We should be able to insert.
        {
            await insertSyntheticTokenConfig(signer0, dataStoreAddress, newFakeToken, 6, BTC_FEED, 60, 3);
            const config = await getTokenConfig(dataStoreAddress, newFakeToken);
            expect(config.enabled).true;
            expect(config.priceFeed).eqls(BTC_FEED);
            expect(config.heartbeatDuration).equals(60);
            expect(config.precision).equals(3);
            expect(config.tokenDecimals).equals(6);
        }
    });
});
