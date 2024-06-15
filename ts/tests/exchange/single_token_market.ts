import { LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { createMarket, executeDeposit, executeOrder, executeWithdrawal } from "../../utils/exchange";
import { SOL_TOKEN_MINT } from "../../utils/token";
import { closeAccount, createAssociatedTokenAccount, createSyncNativeInstruction, getAccount, getAssociatedTokenAddress, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { findPositionPDA, invokeCreateDecreaseOrderWithPayerAsSigner, invokeCreateDepositWithPayerAsSigner, invokeCreateIncreaseOrderWithPayerAsSigner, invokeCreateWithdrawalWithPayerAsSigner } from "gmsol";
import { toInteger } from "lodash";

describe("exchange: single token market", () => {
    const provider = getProvider();
    const { signer0, user0 } = getUsers();
    const { dataStore, exchange } = getPrograms();

    let store: PublicKey;
    let tokenMap: PublicKey;
    let GMWsolWsolWsol: PublicKey;
    let user0WsolTokenAccount: PublicKey;
    let oracleAddress: PublicKey;
    let user0GMWsolAccount: PublicKey;
    before(async () => {
        ({
            dataStoreAddress: store,
            user0WsolTokenAccount,
            oracleAddress,
        } = await getAddresses());
        tokenMap = (await dataStore.account.store.fetch(store)).tokenMap;

        // Initialize WSOL single token market.
        GMWsolWsolWsol = await createMarket(signer0, "WSOL", store, SOL_TOKEN_MINT, SOL_TOKEN_MINT, SOL_TOKEN_MINT, true);

        // Inititalize GM account.
        user0GMWsolAccount = await createAssociatedTokenAccount(provider.connection, user0, GMWsolWsolWsol, user0.publicKey);
        console.log(`Initialized WOL single token GM account: ${user0GMWsolAccount}`);

        // Wrap some SOL for user0.
        const tx = new Transaction().add(
            SystemProgram.transfer({
                fromPubkey: user0.publicKey,
                toPubkey: user0WsolTokenAccount,
                lamports: 1.5 * LAMPORTS_PER_SOL,
            }),
            createSyncNativeInstruction(user0WsolTokenAccount),
        );
        const signature = await provider.sendAndConfirm(tx, [user0]);
        console.log(`wrapped SOL at ${signature}`);
    });

    it("deposit into the single WSOL market", async () => {
        let deposit: PublicKey;
        try {
            const [signature, address] = await invokeCreateDepositWithPayerAsSigner(exchange, {
                store,
                payer: user0,
                marketToken: GMWsolWsolWsol,
                initialLongToken: SOL_TOKEN_MINT,
                initialShortToken: SOL_TOKEN_MINT,
                initialLongTokenAmount: 500_000_000,
                initialShortTokenAmount: 500_000_000,
                options: {
                    tokenMap,
                }
            }, {
                computeUnits: 400_000,
            });
            console.log(`deposit created at ${signature}`);
            deposit = address;
        } catch (error) {
            console.log(error);
        }
        try {
            const [signature] = await executeDeposit(false, provider.connection, {
                authority: signer0,
                store,
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
        }
    });

    it("increase and decrease position", async () => {
        const recentTimestamp = toInteger(Date.now() / 1000);
        // Increase position.
        let increaseOrder: PublicKey;
        const size = 4_000_000_000_000_000_000_000n;
        try {
            const [signature, address] = await invokeCreateIncreaseOrderWithPayerAsSigner(exchange, {
                store,
                payer: user0,
                marketToken: GMWsolWsolWsol,
                collateralToken: SOL_TOKEN_MINT,
                initialCollateralDeltaAmount: 100_000_000,
                isLong: false,
                sizeDeltaUsd: size,
                options: {
                    initialCollateralToken: SOL_TOKEN_MINT,
                    hint: {
                        longToken: SOL_TOKEN_MINT,
                        shortToken: SOL_TOKEN_MINT,
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
                store,
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
            const [position] = findPositionPDA(store, user0.publicKey, GMWsolWsolWsol, SOL_TOKEN_MINT, false);
            const [signature, address] = await invokeCreateDecreaseOrderWithPayerAsSigner(exchange, {
                store,
                payer: user0,
                position,
                initialCollateralDeltaAmount: 0,
                sizeDeltaUsd: size,
                options: {
                    finalOutputToken: SOL_TOKEN_MINT,
                    hint: {
                        market: {
                            marketToken: GMWsolWsolWsol,
                            longToken: SOL_TOKEN_MINT,
                            shortToken: SOL_TOKEN_MINT,
                        },
                        collateralToken: SOL_TOKEN_MINT,
                        isLong: false,
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
                store,
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

    it("withdraw from single WSOL market", async () => {
        let withdrawal: PublicKey;
        let amount = (await getAccount(provider.connection, user0GMWsolAccount)).amount;
        // Create again.
        try {
            const [signature, address] = await invokeCreateWithdrawalWithPayerAsSigner(
                exchange,
                {
                    store,
                    payer: user0,
                    marketToken: GMWsolWsolWsol,
                    amount,
                    finalLongToken: SOL_TOKEN_MINT,
                    finalShortToken: SOL_TOKEN_MINT,
                    options: {
                        tokenMap,
                    }
                }
            );
            console.log(`withdrawal of amount ${amount} created at ${signature}`);
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
            const signature = await executeWithdrawal(
                false,
                provider.connection,
                {
                    authority: signer0,
                    store,
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

    after(async () => {
        // Unwrap the SOL of user0.
        const signature = await closeAccount(provider.connection, user0, user0WsolTokenAccount, user0.publicKey, user0);
        console.log(`unwrapped SOL at ${signature}`);
    });
});
