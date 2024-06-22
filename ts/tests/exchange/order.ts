import { PublicKey } from "@solana/web3.js";
import { getAddresses, getMarkets, getProvider, getUsers } from "../../utils/fixtures";
import { exchange, executeOrder } from "../../utils/exchange";
import { findPositionPDA, invokeCreateDecreaseOrderWithPayerAsSigner, invokeCreateIncreaseOrderWithPayerAsSigner } from "gmsol";
import { toInteger } from "lodash";
import { dataStore } from "../../utils/data";

describe("exchange: order", () => {
    const provider = getProvider();
    const { signer0, user0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let tokenMap: PublicKey;
    let oracleAddress: PublicKey;
    let user0FakeTokenAccount: PublicKey;
    let user0UsdGTokenAccount: PublicKey;
    let GMFakeFakeUsdG: PublicKey;
    let usdGTokenMint: PublicKey;
    let fakeTokenMint: PublicKey;

    before(async () => {
        ({
            dataStoreAddress,
            oracleAddress,
            user0FakeTokenAccount,
            user0UsdGTokenAccount,
            usdGTokenMint,
            fakeTokenMint,
        } = await getAddresses());
        ({ GMFakeFakeUsdG } = await getMarkets());
        tokenMap = (await dataStore.account.store.fetch(dataStoreAddress)).tokenMap;
    });

    it("increase and decrease position", async () => {
        const recentTimestamp = toInteger(Date.now() / 1000);
        // Increase position.
        let increaseOrder: PublicKey;
        try {
            const [signature, address] = await invokeCreateIncreaseOrderWithPayerAsSigner(exchange, {
                store: dataStoreAddress,
                payer: user0,
                marketToken: GMFakeFakeUsdG,
                collateralToken: usdGTokenMint,
                initialCollateralDeltaAmount: 2_000_000,
                isLong: true,
                sizeDeltaUsd: 200_000_000_000_000_000_000n,
                options: {
                    initialCollateralToken: fakeTokenMint,
                    swapPath: [
                        GMFakeFakeUsdG
                    ],
                    hint: {
                        longToken: fakeTokenMint,
                        shortToken: usdGTokenMint,
                    },
                    tokenMap,
                }
            }, {
                computeUnits: 400_000,
            });
            increaseOrder = address;
            console.log(`order ${increaseOrder} created at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }
        try {
            const signature = await executeOrder(false, provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                oracle: oracleAddress,
                order: increaseOrder,
                recentTimestamp,
                holding: dataStore.provider.publicKey,
                options: {
                    executionFee: 5001,
                }
            }, {
                computeUnits: 400_000,
            });
            console.log(`order ${increaseOrder} executed at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }

        // Decrease position.
        let decreaseOrder: PublicKey;
        try {
            const [position] = findPositionPDA(dataStoreAddress, user0.publicKey, GMFakeFakeUsdG, usdGTokenMint, true);
            const [signature, address] = await invokeCreateDecreaseOrderWithPayerAsSigner(exchange, {
                store: dataStoreAddress,
                payer: user0,
                position,
                initialCollateralDeltaAmount: 0,
                sizeDeltaUsd: 200_000_000_000_000_000_000n,
                options: {
                    finalOutputToken: fakeTokenMint,
                    swapPath: [
                        GMFakeFakeUsdG
                    ],
                    hint: {
                        market: {
                            marketToken: GMFakeFakeUsdG,
                            longToken: fakeTokenMint,
                            shortToken: usdGTokenMint,
                        },
                        collateralToken: usdGTokenMint,
                        isLong: true,
                    },
                    tokenMap,
                }
            });
            decreaseOrder = address;
            console.log(`order ${decreaseOrder} created at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }
        try {
            const signature = await executeOrder(false, provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                oracle: oracleAddress,
                order: decreaseOrder,
                recentTimestamp,
                holding: dataStore.provider.publicKey,
                options: {
                    executionFee: 5001,
                }
            }, {
                computeUnits: 800_000,
            });
            console.log(`order ${decreaseOrder} executed at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }
    });

    it("decrease position without swap", async () => {
        const recentTimestamp = toInteger(Date.now() / 1000);
        // Increase position.
        let increaseOrder: PublicKey;
        try {
            const [signature, address] = await invokeCreateIncreaseOrderWithPayerAsSigner(exchange, {
                store: dataStoreAddress,
                payer: user0,
                marketToken: GMFakeFakeUsdG,
                collateralToken: fakeTokenMint,
                initialCollateralDeltaAmount: 400_000_000,
                isLong: true,
                sizeDeltaUsd: 200_000_000_000_000_000_000n,
                options: {
                    initialCollateralToken: usdGTokenMint,
                    swapPath: [
                        GMFakeFakeUsdG
                    ],
                    hint: {
                        longToken: fakeTokenMint,
                        shortToken: usdGTokenMint,
                    },
                    tokenMap,
                }
            }, {
                computeUnits: 400_000,
            });
            increaseOrder = address;
            console.log(`order ${increaseOrder} created at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }
        try {
            const signature = await executeOrder(false, provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                oracle: oracleAddress,
                order: increaseOrder,
                recentTimestamp,
                holding: dataStore.provider.publicKey,
                options: {
                    executionFee: 5001,
                }
            }, {
                computeUnits: 400_000,
            });
            console.log(`order ${increaseOrder} executed at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }

        // Decrease position.
        let decreaseOrder: PublicKey;
        try {
            const [position] = findPositionPDA(dataStoreAddress, user0.publicKey, GMFakeFakeUsdG, fakeTokenMint, true);
            const [signature, address] = await invokeCreateDecreaseOrderWithPayerAsSigner(exchange, {
                store: dataStoreAddress,
                payer: user0,
                position,
                initialCollateralDeltaAmount: 0,
                sizeDeltaUsd: 200_000_000_000_000_000_000n,
                options: {
                    finalOutputToken: fakeTokenMint,
                    swapPath: [],
                    hint: {
                        market: {
                            marketToken: GMFakeFakeUsdG,
                            longToken: fakeTokenMint,
                            shortToken: usdGTokenMint,
                        },
                        collateralToken: fakeTokenMint,
                        isLong: true,
                    },
                    tokenMap,
                }
            });
            decreaseOrder = address;
            console.log(`order ${decreaseOrder} created at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }
        try {
            const signature = await executeOrder(false, provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                oracle: oracleAddress,
                order: decreaseOrder,
                recentTimestamp,
                holding: dataStore.provider.publicKey,
                options: {
                    executionFee: 5001,
                }
            }, {
                computeUnits: 800_000,
            });
            console.log(`order ${decreaseOrder} executed at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }
    });
});
