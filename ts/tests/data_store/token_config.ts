import { Keypair, PublicKey } from '@solana/web3.js';

import { expect, getAddresses, getProvider, getUsers } from "../../utils/fixtures";
import { MARKET_KEEPER, createDataStorePDA, createRolesPDA, dataStore, extendTokenConfigMap, getTokenConfig, insertSyntheticTokenConfig, insertTokenConfig, invokeInitializeTokenMap, invokeInsertTokenConfigAmount, invokePushToTokenMap, invokeSetTokenMap, setExpectedProvider, toggleTokenConfig } from "../../utils/data";
import { AnchorError } from '@coral-xyz/anchor';
import { createMint } from '@solana/spl-token';
import { BTC_FEED, SOL_FEED } from '../../utils/token';
import { PriceProvider } from 'gmsol';

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
        await expect(insertTokenConfig(user0, dataStoreAddress, fakeTokenMint, 123, 10, {})).to.be.rejectedWith(AnchorError, "Permission denied");
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
            await insertTokenConfig(signer0, dataStoreAddress, newToken, 60, 3, {
                chainlinkFeed: BTC_FEED,
                expectedProvider: PriceProvider.Chainlink,
            });
            const config = await getTokenConfig(dataStoreAddress, newToken);
            expect(config.enabled).true;
            expect(config.feeds[1]).eqls(BTC_FEED);
            expect(config.heartbeatDuration).equals(60);
            expect(config.precision).equals(3);
        }

        // Update the config by inserting again.
        {
            await insertTokenConfig(signer0, dataStoreAddress, newToken, 42, 5, {
                chainlinkFeed: SOL_FEED,
            });
            const config = await getTokenConfig(dataStoreAddress, newToken);
            expect(config.enabled).true;
            expect(config.feeds[1]).eqls(SOL_FEED);
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

        // Select a different expected provider.
        {
            await setExpectedProvider(signer0, dataStoreAddress, newToken, PriceProvider.PythLegacy);
            const config = await getTokenConfig(dataStoreAddress, newToken);
            expect(config.expectedProvider).eqls(PriceProvider.PythLegacy);
        }

        // Insert timestamp adjustment.
        {
            await invokeInsertTokenConfigAmount(dataStore, {
                authority: signer0,
                store: dataStoreAddress,
                token: newToken,
                key: 'timestamp_adjustment',
                amount: 3
            });
            console.log(`insert an amount for ${newToken.toBase58()}`);
        }
    });

    it("insert synthetic token", async () => {
        const newFakeToken = PublicKey.unique();
        // We should be able to insert.
        {
            await insertSyntheticTokenConfig(signer0, dataStoreAddress, newFakeToken, 6, 60, 3, {
                chainlinkFeed: BTC_FEED,
                expectedProvider: PriceProvider.Chainlink,
            });
            const config = await getTokenConfig(dataStoreAddress, newFakeToken);
            expect(config.enabled).true;
            expect(config.feeds[1]).eqls(BTC_FEED);
            expect(config.heartbeatDuration).equals(60);
            expect(config.precision).equals(3);
            expect(config.tokenDecimals).equals(6);
        }
    });

    it("initialize a new token map and set to a new store", async () => {
        const randomKey = Keypair.generate().publicKey.toBase58().slice(0, 10);
        const [store] = createDataStorePDA(randomKey);
        await dataStore.methods.initialize(randomKey).accounts({
            authority: provider.publicKey,
            dataStore: store,
        }).rpc();
        await dataStore.methods.enableRole(MARKET_KEEPER).accounts({
            authority: provider.publicKey,
            store,
        }).rpc();
        console.log(`initialized a new store ${store} and enabled MARKET_KEEPER role`);

        const beforeSet = await dataStore.methods.getTokenMap().accounts({ store }).view();
        expect(beforeSet).null;

        const tokenMap = Keypair.generate();
        await invokeInitializeTokenMap(dataStore, {
            payer: signer0,
            store,
            tokenMap,
        });
        console.log(`initialized a new token map ${tokenMap.publicKey}`);

        // Only MARKET_KEEPER can set token map.
        await expect(invokeSetTokenMap(dataStore, {
            authority: signer0,
            store,
            tokenMap: tokenMap.publicKey,
        })).rejectedWith(Error, "Permission denied");

        await dataStore.methods.grantRole(signer0.publicKey, MARKET_KEEPER).accounts({
            authority: provider.publicKey,
            store,
        }).rpc();

        await invokeSetTokenMap(dataStore, {
            authority: signer0,
            store,
            tokenMap: tokenMap.publicKey,
        });

        const afterSet = await dataStore.methods.getTokenMap().accounts({ store }).view();
        expect(tokenMap.publicKey.equals(afterSet));

        const beforeSize = (await dataStore.account.tokenMapHeader.getAccountInfo(tokenMap.publicKey)).data.byteLength;
        console.log(`size before the push: ${beforeSize}`);
        await invokePushToTokenMap(dataStore, {
            authority: signer0,
            store,
            tokenMap: tokenMap.publicKey,
            token: fakeTokenMint,
            heartbeatDuration: 120,
            precision: 4,
            feeds: {
                chainlinkFeed: BTC_FEED,
                expectedProvider: PriceProvider.Chainlink,
            }
        }, {
            skipPreflight: true,
        });
        const afterSize = (await dataStore.account.tokenMapHeader.getAccountInfo(tokenMap.publicKey)).data.byteLength;
        console.log(`size after the push: ${afterSize}`);
    });
});
