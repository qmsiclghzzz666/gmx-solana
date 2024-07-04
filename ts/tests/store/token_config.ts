import { Keypair, PublicKey } from '@solana/web3.js';

import { expect, getAddresses, getProvider, getUsers } from "../../utils/fixtures";
import { MARKET_KEEPER, createDataStorePDA, createRolesPDA, storeProgram, invokeInitializeTokenMap, invokePushToTokenMap, invokePushToTokenMapSynthetic, invokeSetFeedConfig, invokeSetTokenMap, setExpectedProvider, toggleTokenConfig } from "../../utils/data";
import { AnchorError } from '@coral-xyz/anchor';
import { createMint } from '@solana/spl-token';
import { BTC_FEED, SOL_FEED, SOL_FEED_PYTH } from '../../utils/token';
import { PriceProvider } from 'gmsol';

describe("store: TokenConfig", () => {
    const provider = getProvider();
    const { signer0, user0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let roles: PublicKey;
    let fakeTokenMint: PublicKey;
    let usdGTokenMint: PublicKey;
    let tokenMap: PublicKey;
    before("init token config", async () => {
        ({ dataStoreAddress, fakeTokenMint, usdGTokenMint } = await getAddresses());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
        tokenMap = (await storeProgram.account.store.fetch(dataStoreAddress)).tokenMap;
    });

    const fakeTokenName = "FAKE";
    const usdGTokenName = "USDG";

    it("can only be updated by MARKET_KEEPER", async () => {
        await expect(invokePushToTokenMap(storeProgram, {
            authority: user0,
            store: dataStoreAddress,
            name: fakeTokenName,
            tokenMap,
            token: fakeTokenMint,
            heartbeatDuration: 123,
            precision: 10,
            feeds: {}
        })).to.be.rejectedWith(Error, "Permission denied");
    });

    it("test token config map", async () => {
        const newToken = await createMint(provider.connection, signer0, signer0.publicKey, signer0.publicKey, 10);

        // Config not exists yet.
        {
            await expect(storeProgram.methods.isTokenConfigEnabled(newToken).accounts({
                tokenMap,
            }).view()).rejected;
        }

        // We should be able to insert.
        {
            await invokePushToTokenMap(storeProgram, {
                authority: signer0,
                store: dataStoreAddress,
                tokenMap,
                name: fakeTokenName,
                token: newToken,
                heartbeatDuration: 60,
                precision: 3,
                feeds: {
                    chainlinkFeed: BTC_FEED,
                    expectedProvider: PriceProvider.Chainlink,
                }
            });
            const enabled = await storeProgram.methods.isTokenConfigEnabled(newToken).accounts({
                tokenMap,
            }).view();
            expect(enabled).true;
            const feed = await storeProgram.methods.tokenFeed(newToken, 1).accounts({ tokenMap }).view();
            expect(BTC_FEED.equals(feed));
        }

        // // Update the config by inserting again.
        // {
        //     await insertTokenConfig(signer0, dataStoreAddress, newToken, 42, 5, {
        //         chainlinkFeed: SOL_FEED,
        //     });
        //     const config = await getTokenConfig(dataStoreAddress, newToken);
        //     expect(config.enabled).true;
        //     expect(config.feeds[1]).eqls(SOL_FEED);
        //     expect(config.heartbeatDuration).equals(42);
        //     expect(config.precision).equals(5);
        // }

        // Issue a full update by inserting again.
        {
            await invokePushToTokenMap(storeProgram, {
                authority: signer0,
                store: dataStoreAddress,
                tokenMap,
                name: fakeTokenName,
                token: newToken,
                heartbeatDuration: 60,
                precision: 3,
                feeds: {
                    chainlinkFeed: SOL_FEED,
                    expectedProvider: PriceProvider.Chainlink,
                },
                update: true,
            });
            const enabled = await storeProgram.methods.isTokenConfigEnabled(newToken).accounts({
                tokenMap,
            }).view();
            expect(enabled).true;
            const feed = await storeProgram.methods.tokenFeed(newToken, 1).accounts({ tokenMap }).view();
            expect(SOL_FEED.equals(feed));
        }

        // We can disable the config temporarily.
        {
            await toggleTokenConfig(signer0, dataStoreAddress, tokenMap, newToken, false);
            const enabled = await storeProgram.methods.isTokenConfigEnabled(newToken).accounts({
                tokenMap,
            }).view();
            expect(enabled).false;
        }

        // And we can enable the config again.
        {
            await toggleTokenConfig(signer0, dataStoreAddress, tokenMap, newToken, true);
            const enabled = await storeProgram.methods.isTokenConfigEnabled(newToken).accounts({
                tokenMap,
            }).view();
            expect(enabled).true;
        }

        // Select a different expected provider.
        {
            await setExpectedProvider(signer0, dataStoreAddress, tokenMap, newToken, PriceProvider.PythLegacy);
            const expectedProvider = await storeProgram.methods.tokenExpectedProvider(newToken).accounts({ tokenMap }).view();
            expect(expectedProvider).eqls(PriceProvider.PythLegacy);
        }

        // // Insert timestamp adjustment.
        // {
        //     await invokeInsertTokenConfigAmount(dataStore, {
        //         authority: signer0,
        //         store: dataStoreAddress,
        //         token: newToken,
        //         key: 'timestamp_adjustment',
        //         amount: 3
        //     });
        //     console.log(`insert an amount for ${newToken.toBase58()}`);
        // }
    });

    it("insert synthetic token", async () => {
        const newFakeToken = PublicKey.unique();
        // We should be able to insert.
        {
            await invokePushToTokenMapSynthetic(storeProgram, {
                authority: signer0,
                store: dataStoreAddress,
                tokenMap,
                name: fakeTokenName,
                token: newFakeToken,
                tokenDecimals: 6,
                heartbeatDuration: 60,
                precision: 3,
                feeds: {
                    chainlinkFeed: BTC_FEED,
                    expectedProvider: PriceProvider.Chainlink,
                }
            });
            const enabled = await storeProgram.methods.isTokenConfigEnabled(newFakeToken).accounts({
                tokenMap,
            }).view();
            expect(enabled).true;
            const feed = await storeProgram.methods.tokenFeed(newFakeToken, 1).accounts({ tokenMap }).view();
            expect(BTC_FEED.equals(feed));
        }
    });

    it("initialize a new token map and set to a new store", async () => {
        const randomKey = Keypair.generate().publicKey.toBase58().slice(0, 10);
        const [store] = createDataStorePDA(randomKey);
        await storeProgram.methods.initialize(randomKey, null).accounts({
            payer: provider.publicKey,
            store,
        }).rpc();
        await storeProgram.methods.enableRole(MARKET_KEEPER).accounts({
            authority: provider.publicKey,
            store,
        }).rpc();
        console.log(`initialized a new store ${store} and enabled MARKET_KEEPER role`);

        const beforeSet = await storeProgram.methods.getTokenMap().accounts({ store }).view();
        expect(beforeSet).null;

        const tokenMap = Keypair.generate();
        await invokeInitializeTokenMap(storeProgram, {
            payer: signer0,
            store,
            tokenMap,
        });
        console.log(`initialized a new token map ${tokenMap.publicKey}`);

        // Only MARKET_KEEPER can set token map.
        await expect(invokeSetTokenMap(storeProgram, {
            authority: signer0,
            store,
            tokenMap: tokenMap.publicKey,
        })).rejectedWith(Error, "Permission denied");

        await storeProgram.methods.grantRole(signer0.publicKey, MARKET_KEEPER).accounts({
            authority: provider.publicKey,
            store,
        }).rpc();

        await invokeSetTokenMap(storeProgram, {
            authority: signer0,
            store,
            tokenMap: tokenMap.publicKey,
        });

        const afterSet = await storeProgram.methods.getTokenMap().accounts({ store }).view();
        expect(tokenMap.publicKey.equals(afterSet));
        const beforeSize = (await storeProgram.account.tokenMapHeader.getAccountInfo(tokenMap.publicKey)).data.byteLength;
        console.log(`size before the push: ${beforeSize}`);

        {
            await invokePushToTokenMap(storeProgram, {
                authority: signer0,
                store,
                name: fakeTokenName,
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
            const afterSize = (await storeProgram.account.tokenMapHeader.getAccountInfo(tokenMap.publicKey)).data.byteLength;
            console.log(`size after the push: ${afterSize}`);
            const feed = await storeProgram.methods.tokenFeed(fakeTokenMint, PriceProvider.Chainlink).accounts({ tokenMap: tokenMap.publicKey }).view();
            expect(BTC_FEED.equals(feed));
        }

        {
            await expect(invokePushToTokenMap(storeProgram, {
                authority: signer0,
                store,
                tokenMap: tokenMap.publicKey,
                name: fakeTokenName,
                token: fakeTokenMint,
                heartbeatDuration: 120,
                precision: 4,
                feeds: {
                    chainlinkFeed: BTC_FEED,
                    expectedProvider: PriceProvider.Chainlink,
                }
            }, {
                skipPreflight: false,
            })).rejectedWith(Error, "Aready exist");
        }

        {
            await invokePushToTokenMap(storeProgram, {
                authority: signer0,
                store,
                tokenMap: tokenMap.publicKey,
                name: usdGTokenName,
                token: usdGTokenMint,
                heartbeatDuration: 120,
                precision: 4,
                feeds: {
                    chainlinkFeed: SOL_FEED,
                    expectedProvider: PriceProvider.Chainlink,
                }
            }, {
                skipPreflight: true,
            });
            const afterSize = (await storeProgram.account.tokenMapHeader.getAccountInfo(tokenMap.publicKey)).data.byteLength;
            console.log(`size after the push: ${afterSize}`);
        }
        const fakeFeed = await storeProgram.methods.tokenFeed(fakeTokenMint, PriceProvider.Chainlink).accounts({ tokenMap: tokenMap.publicKey }).view();
        expect(BTC_FEED.equals(fakeFeed));
        const usdGFeed = await storeProgram.methods.tokenFeed(usdGTokenMint, PriceProvider.Chainlink).accounts({ tokenMap: tokenMap.publicKey }).view();
        expect(SOL_FEED.equals(usdGFeed));

        {
            // Can only be set by MARKET_KEEPER.
            await expect(invokeSetFeedConfig(storeProgram, {
                authority: user0,
                store,
                tokenMap: tokenMap.publicKey,
                token: usdGTokenMint,
                provider: PriceProvider.PythLegacy,
                feed: SOL_FEED_PYTH,
                timestampAdjustment: 3,
            })).rejectedWith(Error, "Permission denied");
        }

        {
            const tx = await invokeSetFeedConfig(storeProgram, {
                authority: signer0,
                store,
                tokenMap: tokenMap.publicKey,
                token: usdGTokenMint,
                provider: PriceProvider.PythLegacy,
                feed: SOL_FEED_PYTH,
                timestampAdjustment: 42,
            });
            console.log(`set feed config at tx ${tx}`);

            const usdGFeed = await storeProgram.methods.tokenFeed(usdGTokenMint, PriceProvider.PythLegacy).accounts({ tokenMap: tokenMap.publicKey }).view();
            expect(SOL_FEED_PYTH.equals(usdGFeed));
            const timestmapAdjustment = await storeProgram.methods.tokenTimestampAdjustment(usdGTokenMint, PriceProvider.PythLegacy).accounts({ tokenMap: tokenMap.publicKey }).view();
            expect(timestmapAdjustment).eqls(42);
        }

    });
});
