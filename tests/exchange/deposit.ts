import { PublicKey } from "@solana/web3.js";
import { createMarketPDA, createNoncePDA, createRolesPDA } from "../../utils/data";
import { getAddresses, getMarkets, getPrograms, getProvider, getUsers, expect } from "../../utils/fixtures";
import { invokeCancelDeposit, invokeCancelWithdrawal, invokeExecuteWithdrawal, invokeCreateDeposit, invokeExecuteDeposit, invokeCreateWithdrawal } from "../../utils/exchange";
import { AnchorError } from "@coral-xyz/anchor";

describe("exchange: deposit", () => {
    const provider = getProvider();
    const { dataStore } = getPrograms();
    const { signer0, user0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let user0FakeTokenAccount: PublicKey;
    let user0UsdGTokenAccount: PublicKey;
    let user0WsolTokenAccount: PublicKey;
    let user0WsolWsolUsdGTokenAccount: PublicKey;
    let user0FakeFakeUsdGTokenAccount: PublicKey;
    let GMFakeFakeUsdG: PublicKey;
    let GMWsolWsolUsdG: PublicKey;
    let oracleAddress: PublicKey;

    before(async () => {
        ({
            dataStoreAddress,
            user0FakeTokenAccount,
            user0UsdGTokenAccount,
            user0FakeFakeUsdGTokenAccount,
            oracleAddress,
            user0WsolTokenAccount,
            user0WsolWsolUsdGTokenAccount,
        } = await getAddresses());
        ({ GMFakeFakeUsdG, GMWsolWsolUsdG } = await getMarkets());
    });

    it("create and execute deposit and then withdraw", async () => {
        let deposit: PublicKey;
        try {
            const [signature, address] = await invokeCreateDeposit(provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                payer: user0,
                marketToken: GMFakeFakeUsdG,
                toMarketTokenAccount: user0FakeFakeUsdGTokenAccount,
                fromInitialLongTokenAccount: user0FakeTokenAccount,
                fromInitialShortTokenAccount: user0UsdGTokenAccount,
                initialLongTokenAmount: 1_000_000_000,
                initialShortTokenAmount: 70_000 * 100_000_000,
            });
            console.log(`deposit created at ${signature}`);
            deposit = address;
        } catch (error) {
            console.log(error);
        }
        try {
            const [signature] = await invokeExecuteDeposit(provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                oracle: oracleAddress,
                deposit,
                options: {
                    executionFee: 5_001,
                }
            }, {
                computeUnits: 800_000,
            });
            console.log(`deposit executed at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        } finally {
            const afterExecution = await dataStore.account.oracle.fetch(oracleAddress);
            expect(afterExecution.primary.prices.length).equals(0);
            // const market = await dataStore.account.market.fetch(marketFakeFakeUsdG);
            // console.log("pools", market.pools);
        }

        let withdrawal: PublicKey;
        try {
            const [signature, withdrawalAddress] = await invokeCreateWithdrawal(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    payer: user0,
                    marketToken: GMFakeFakeUsdG,
                    amount: 1_000_000_000_000,
                    fromMarketTokenAccount: user0FakeFakeUsdGTokenAccount,
                    toLongTokenAccount: user0FakeTokenAccount,
                    toShortTokenAccount: user0UsdGTokenAccount
                }
            );
            console.log(`withdrawal created at ${signature}`);
            withdrawal = withdrawalAddress;
        } catch (error) {
            console.log(error);
            throw error;
        } finally {
            const afterExecution = await dataStore.account.oracle.fetch(oracleAddress);
            expect(afterExecution.primary.prices.length).equals(0);
            // const market = await dataStore.account.market.fetch(marketFakeFakeUsdG);
            // console.log("pools", market.pools);
        }
        // Cancel the withdrawal.
        try {
            const [signature] = await invokeCancelWithdrawal(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    withdrawal,
                    options: {
                        executionFee: 5000,
                    }
                }
            );
            console.log(`withdrawal cancelled at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        } finally {
            const afterExecution = await dataStore.account.oracle.fetch(oracleAddress);
            expect(afterExecution.primary.prices.length).equals(0);
            // const market = await dataStore.account.market.fetch(marketFakeFakeUsdG);
            // console.log("pools", market.pools);
        }
        // Create again.
        try {
            const [signature, address] = await invokeCreateWithdrawal(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    payer: user0,
                    marketToken: GMFakeFakeUsdG,
                    amount: 2_000 * 1_000_000_000,
                    fromMarketTokenAccount: user0FakeFakeUsdGTokenAccount,
                    toLongTokenAccount: user0FakeTokenAccount,
                    toShortTokenAccount: user0UsdGTokenAccount,
                }
            );
            console.log(`withdrawal created at ${signature}`);
            withdrawal = address;
        } catch (error) {
            console.log(error);
            throw error;
        } finally {
            const afterExecution = await dataStore.account.oracle.fetch(oracleAddress);
            expect(afterExecution.primary.prices.length).equals(0);
            // const market = await dataStore.account.market.fetch(marketFakeFakeUsdG);
            // console.log("pools", market.pools);
        }
        try {
            const signature = await invokeExecuteWithdrawal(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    oracle: oracleAddress,
                    withdrawal,
                    options: {
                        executionFee: 5001,
                    }
                },
                {
                    computeUnits: 400_000,
                },
            );
            console.log(`withdrawal executed at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        } finally {
            const afterExecution = await dataStore.account.oracle.fetch(oracleAddress);
            expect(afterExecution.primary.prices.length).equals(0);
            // const market = await dataStore.account.market.fetch(marketFakeFakeUsdG);
            // console.log("pools", market.pools);
        }
    });

    it("create and cancel deposit", async () => {
        const [signature, deposit] = await invokeCreateDeposit(
            provider.connection,
            {
                authority: signer0,
                store: dataStoreAddress,
                payer: user0,
                marketToken: GMFakeFakeUsdG,
                toMarketTokenAccount: user0FakeFakeUsdGTokenAccount,
                fromInitialLongTokenAccount: user0FakeTokenAccount,
                fromInitialShortTokenAccount: user0UsdGTokenAccount,
                initialLongTokenAmount: 2_000_000_000,
                initialShortTokenAmount: 200_000_000,
            }
        );
        console.log(`deposit created at ${signature}`);
        try {
            const signature = await invokeCancelDeposit(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    deposit,
                    options: {
                        executionFee: 5000,
                    }
                }
            );
            console.log(`deposit cancelled at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        } finally {
            const afterExecution = await dataStore.account.oracle.fetch(oracleAddress);
            expect(afterExecution.primary.prices.length).equals(0);
            // const market = await dataStore.account.market.fetch(marketFakeFakeUsdG);
            // console.log("pools", market.pools);
        }
    });

    it("create deposit with invalid swap path", async () => {
        await expect(invokeCreateDeposit(
            provider.connection,
            {
                authority: signer0,
                store: dataStoreAddress,
                payer: user0,
                marketToken: GMFakeFakeUsdG,
                toMarketTokenAccount: user0FakeFakeUsdGTokenAccount,
                fromInitialLongTokenAccount: user0FakeTokenAccount,
                fromInitialShortTokenAccount: user0UsdGTokenAccount,
                initialLongTokenAmount: 2_000_000_000,
                initialShortTokenAmount: 200_000_000,
                options: {
                    longTokenSwapPath: [GMFakeFakeUsdG],
                }
            }
        )).rejectedWith(AnchorError, "Invalid swap path");
    });

    it("create withdrawal with invalid swap path", async () => {
        await expect(invokeCreateWithdrawal(
            provider.connection,
            {
                authority: signer0,
                store: dataStoreAddress,
                payer: user0,
                marketToken: GMFakeFakeUsdG,
                amount: 1_000,
                fromMarketTokenAccount: user0FakeFakeUsdGTokenAccount,
                toLongTokenAccount: user0FakeTokenAccount,
                toShortTokenAccount: user0UsdGTokenAccount,
                options: {
                    longTokenSwapPath: [GMFakeFakeUsdG],
                }
            }
        )).rejectedWith(AnchorError, "Invalid swap path");
    });

    it("create withdrawal with valid swap path and cancel", async () => {
        let withdrawal: PublicKey;
        {
            const [signature, withdrawAddress] = await invokeCreateWithdrawal(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    payer: user0,
                    marketToken: GMFakeFakeUsdG,
                    amount: 1_000_000,
                    fromMarketTokenAccount: user0FakeFakeUsdGTokenAccount,
                    toLongTokenAccount: user0UsdGTokenAccount,
                    toShortTokenAccount: user0UsdGTokenAccount,
                    options: {
                        longTokenSwapPath: [GMFakeFakeUsdG],
                    }
                }
            );
            withdrawal = withdrawAddress;
            console.log(`withdrawal created at ${signature}`);
        }

        {
            const [signature] = await invokeCancelWithdrawal(provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                withdrawal,
            });
            console.log(`withdrawal cancelled at ${signature}`);
        }
    });

    it("create deposit with valid swap path and cancel", async () => {
        let deposit: PublicKey;
        {
            const [signature, depositAddress] = await invokeCreateDeposit(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    payer: user0,
                    marketToken: GMWsolWsolUsdG,
                    toMarketTokenAccount: user0WsolWsolUsdGTokenAccount,
                    fromInitialShortTokenAccount: user0FakeTokenAccount,
                    initialShortTokenAmount: 1_000_000,
                    options: {
                        shortTokenSwapPath: [GMFakeFakeUsdG],
                    }
                }
            );
            deposit = depositAddress;
            console.log(`deposit created at ${signature}`);
        }

        {
            const [signature] = await invokeCancelDeposit(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    deposit,
                }
            );
            console.log(`deposit cancelled at ${signature}`);
        }
    });

    it("create deposit with valid swap path and execute", async () => {
        const market1 = createMarketPDA(dataStoreAddress, GMWsolWsolUsdG)[0];
        const market2 = createMarketPDA(dataStoreAddress, GMFakeFakeUsdG)[0];
        let deposit: PublicKey;
        try {
            const pool2 = (await dataStore.account.market.fetch(market2)).pools.pools[0];
            console.log(`${pool2.longTokenAmount}:${pool2.shortTokenAmount}`);
            const [signature, depositAddress] = await invokeCreateDeposit(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    payer: user0,
                    marketToken: GMWsolWsolUsdG,
                    toMarketTokenAccount: user0WsolWsolUsdGTokenAccount,
                    fromInitialLongTokenAccount: user0WsolTokenAccount,
                    fromInitialShortTokenAccount: user0FakeTokenAccount,
                    initialLongTokenAmount: 3_000_000,
                    initialShortTokenAmount: 20_000_000,
                    options: {
                        shortTokenSwapPath: [GMFakeFakeUsdG],
                    }
                }
            );
            deposit = depositAddress;
            console.log(`deposit created at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        }

        try {
            const [signature] = await invokeExecuteDeposit(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    oracle: oracleAddress,
                    deposit,
                    options: {
                        executionFee: 5001,
                    }
                },
                {
                    computeUnits: 400_000,
                    skipPreflight: true,
                },
            );
            console.log(`deposit executed at ${signature}`);
        } catch (error) {
            console.log(error);
            throw error;
        } finally {
            const pool1 = (await dataStore.account.market.fetch(market1)).pools.pools[0];
            console.log(`${pool1.longTokenAmount}:${pool1.shortTokenAmount}`);
            const pool2 = (await dataStore.account.market.fetch(market2)).pools.pools[0];
            console.log(`${pool2.longTokenAmount}:${pool2.shortTokenAmount}`);
        }
    });

    it("create withdrawal with valid swap path and execute", async () => {
        let withdrawal: PublicKey;
        {
            const [signature, withdrawAddress] = await invokeCreateWithdrawal(
                provider.connection,
                {
                    authority: signer0,
                    store: dataStoreAddress,
                    payer: user0,
                    marketToken: GMFakeFakeUsdG,
                    amount: 1_000_000,
                    fromMarketTokenAccount: user0FakeFakeUsdGTokenAccount,
                    toLongTokenAccount: user0WsolTokenAccount,
                    toShortTokenAccount: user0WsolTokenAccount,
                    options: {
                        longTokenSwapPath: [GMFakeFakeUsdG, GMWsolWsolUsdG],
                        shortTokenSwapPath: [GMWsolWsolUsdG],
                    }
                }
            );
            withdrawal = withdrawAddress;
            console.log(`withdrawal created at ${signature}`);
        }

        {
            const [signature] = await invokeExecuteWithdrawal(provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                oracle: oracleAddress,
                withdrawal,
                options: {
                    executionFee: 5_001,
                }
            }, {
                computeUnits: 400_000,
                skipPreflight: true,
            });
            console.log(`withdrawal executed at ${signature}`);
        }
    });
});
