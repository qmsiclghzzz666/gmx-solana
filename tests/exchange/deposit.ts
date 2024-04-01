import { PublicKey } from "@solana/web3.js";
import { createNoncePDA, createRolesPDA } from "../../utils/data";
import { getAddresses, getMarkets, getPrograms, getProvider, getUsers, expect } from "../../utils/fixtures";
import { cancelDeposit, cancelWithdrawal, createDeposit, createWithdrawal, executeWithdrawal, invokeExecuteDeposit } from "../../utils/exchange";

describe("exchange: deposit", () => {
    const provider = getProvider();
    const { dataStore } = getPrograms();
    const { signer0, user0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let user0FakeTokenAccount: PublicKey;
    let user0UsdGTokenAccount: PublicKey;
    let user0FakeFakeUsdGTokenAccount: PublicKey;
    let marketFakeFakeUsdG: PublicKey;
    let roles: PublicKey;
    let nonce: PublicKey;
    let oracleAddress: PublicKey;

    before(async () => {
        ({
            dataStoreAddress,
            user0FakeTokenAccount,
            user0UsdGTokenAccount,
            user0FakeFakeUsdGTokenAccount,
            oracleAddress,
        } = await getAddresses());
        ({ marketFakeFakeUsdG } = await getMarkets());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
        [nonce] = createNoncePDA(dataStoreAddress);
    });

    it("create and execute deposit and then withdraw", async () => {
        let deposit: PublicKey;
        try {
            deposit = await createDeposit(
                signer0,
                dataStoreAddress,
                user0,
                marketFakeFakeUsdG,
                user0FakeFakeUsdGTokenAccount,
                user0FakeTokenAccount,
                user0UsdGTokenAccount,
                1_000_000_000,
                70_000 * 100_000_000,
                {
                    callback: signature => console.log(`deposit created at ${signature}`),
                }
            );
        } catch (error) {
            console.log(error);
        }
        try {
            const signature = await invokeExecuteDeposit(provider.connection, {
                authority: signer0,
                store: dataStoreAddress,
                oracle: oracleAddress,
                deposit,
                options: {
                    executionFee: 5_001,
                }
            }, 800_000);
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
            withdrawal = await createWithdrawal(
                signer0,
                dataStoreAddress,
                user0,
                marketFakeFakeUsdG,
                1_000_000_000_000,
                user0FakeFakeUsdGTokenAccount,
                user0FakeTokenAccount,
                user0UsdGTokenAccount,
                {
                    callback: tx => console.log("withdrawal created at", tx),
                }
            );
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
            await cancelWithdrawal(
                signer0,
                dataStoreAddress,
                user0.publicKey,
                withdrawal,
                user0FakeFakeUsdGTokenAccount,
                {
                    executionFee: 5001,
                    callback: tx => console.log("withdrawal cancelled at", tx),
                }
            );
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
            withdrawal = await createWithdrawal(
                signer0,
                dataStoreAddress,
                user0,
                marketFakeFakeUsdG,
                2_000 * 1_000_000_000,
                user0FakeFakeUsdGTokenAccount,
                user0FakeTokenAccount,
                user0UsdGTokenAccount,
                {
                    callback: tx => console.log("withdrawal created at", tx),
                }
            );
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
            await executeWithdrawal(
                signer0,
                dataStoreAddress,
                oracleAddress,
                user0.publicKey,
                withdrawal,
                {
                    callback: tx => console.log("withdrawal executed at", tx),
                    executionFee: 5001,
                }
            );
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
        const deposit = await createDeposit(
            signer0,
            dataStoreAddress,
            user0,
            marketFakeFakeUsdG,
            user0FakeFakeUsdGTokenAccount,
            user0FakeTokenAccount,
            user0UsdGTokenAccount,
            2_000_000_000,
            200_000_000,
            {
                callback: sigature => console.log(`deposit created at ${sigature}`),
            }
        );
        try {
            await cancelDeposit(
                signer0,
                dataStoreAddress,
                deposit,
                {
                    executionFee: 5000,
                    callback: signature => console.log(`deposit cancelled at ${signature}`),
                }
            )
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
});
