import { LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { createMarket, executeDeposit, executeOrder, executeWithdrawal, invokeUpdateMarketConfig } from "../../utils/exchange";
import { SOL_TOKEN_MINT } from "../../utils/token";
import { closeAccount, createAssociatedTokenAccount, createSyncNativeInstruction, getAccount, getAssociatedTokenAddress, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { findPositionPDA, invokeCreateDecreaseOrderWithPayerAsSigner, invokeCreateDepositWithPayerAsSigner, invokeCreateIncreaseOrderWithPayerAsSigner, invokeCreateWithdrawalWithPayerAsSigner } from "gmsol";
import { toInteger } from "lodash";
import { deposit, withdraw, wrap } from "../../utils/exchange/action";

describe("exchange: Single Token Market", () => {
    const provider = getProvider();
    const { signer0, user0 } = getUsers();
    const { storeProgram: dataStore, exchangeProgram: exchange } = getPrograms();

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
        await invokeUpdateMarketConfig(dataStore, {
            authority: signer0,
            store,
            marketToken: GMWsolWsolWsol,
            key: "reserve_factor",
            value: 100_000_000_000_000_000_000n,
        });
        await invokeUpdateMarketConfig(dataStore, {
            authority: signer0,
            store,
            marketToken: GMWsolWsolWsol,
            key: "open_interest_reserve_factor",
            value: 95_000_000_000_000_000_000n,
        });

        // Inititalize GM account.
        user0GMWsolAccount = await createAssociatedTokenAccount(provider.connection, user0, GMWsolWsolWsol, user0.publicKey);
        console.log(`Initialized WSOL single token GM account: ${user0GMWsolAccount}`);

        // Wrap some SOL for user0.
        await wrap(provider, user0, 1.5 * LAMPORTS_PER_SOL);
    });

    it("deposit into the single WSOL market", async () => {
        await deposit(
            exchange,
            user0,
            signer0,
            store,
            tokenMap,
            oracleAddress,
            GMWsolWsolWsol,
            SOL_TOKEN_MINT,
            SOL_TOKEN_MINT,
            500_000_000,
            500_000_000,
            {}
        );
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
        let amount = (await getAccount(provider.connection, user0GMWsolAccount)).amount;

        await withdraw(
            exchange,
            user0,
            signer0,
            store,
            tokenMap,
            oracleAddress,
            GMWsolWsolWsol,
            amount,
            SOL_TOKEN_MINT,
            SOL_TOKEN_MINT,
            {
                storeProgram: dataStore,
            }
        );
    });
});
