import { BN, Provider } from "@coral-xyz/anchor";
import { PublicKey, Signer, SystemProgram, Transaction } from "@solana/web3.js";
import { ExchangeProgram, StoreProgram, invokeCreateDepositWithPayerAsSigner, invokeCreateSwapOrderWithPayerAsSigner, invokeCreateWithdrawalWithPayerAsSigner } from "gmsol";
import { executeDeposit, executeOrder, executeWithdrawal } from ".";
import { expect } from "../fixtures";
import { createSyncNativeInstruction, getAssociatedTokenAddress } from "@solana/spl-token";
import { SOL_TOKEN_MINT } from "../token";
import { toInteger } from "lodash";

export const wrap = async (
    provider: Provider,
    authority: Signer,
    lamports: number,
) => {
    const account = await getAssociatedTokenAddress(SOL_TOKEN_MINT, authority.publicKey);
    const tx = new Transaction().add(
        SystemProgram.transfer({
            fromPubkey: authority.publicKey,
            toPubkey: account,
            lamports,
        }),
        createSyncNativeInstruction(account),
    );
    const signature = await provider.sendAndConfirm(tx, [authority]);
    console.log(`wrapped ${lamports} lamports at ${signature}`);
};

export const deposit = async (
    exchangeProgram: ExchangeProgram,
    user: Signer,
    keeper: Signer,
    store: PublicKey,
    tokenMap: PublicKey,
    oracle: PublicKey,
    marketToken: PublicKey,
    initialLongToken: PublicKey,
    initialShortToken: PublicKey,
    initialLongTokenAmount: number | bigint | BN,
    initialShortTokenAmount: number | bigint | BN,
    {
        executionFee = 5_001,
        computeUnits = 400_000,
        storeProgram,
        longTokenSwapPath,
        shortTokenSwapPath,
    }: {
        executionFee?: number | bigint,
        computeUnits?: number,
        storeProgram?: StoreProgram,
        longTokenSwapPath?,
        shortTokenSwapPath?,
    }
) => {
    let deposit: PublicKey;
    try {
        const [signature, address] = await invokeCreateDepositWithPayerAsSigner(exchangeProgram, {
            store,
            payer: user,
            marketToken,
            initialLongToken,
            initialShortToken,
            initialLongTokenAmount,
            initialShortTokenAmount,
            options: {
                tokenMap,
                longTokenSwapPath,
                shortTokenSwapPath,
            }
        }, {
            computeUnits,
        });
        console.log(`deposit created at ${signature}`);
        deposit = address;
    } catch (error) {
        console.log(error);
    }
    try {
        const [signature] = await executeDeposit(false, exchangeProgram.provider.connection, {
            authority: keeper,
            store,
            oracle,
            deposit,
            options: {
                executionFee,
            }
        }, {
            computeUnits,
        });
        console.log(`deposit executed at ${signature}`);
    } catch (error) {
        console.log(error);
        throw error;
    } finally {
        if (storeProgram) {
            const afterExecution = await storeProgram.account.oracle.fetch(oracle);
            expect(afterExecution.primary.prices.length).equals(0);
        }
    }
    return deposit;
};

export const withdraw = async (
    exchangeProgram: ExchangeProgram,
    user: Signer,
    keeper: Signer,
    store: PublicKey,
    tokenMap: PublicKey,
    oracle: PublicKey,
    marketToken: PublicKey,
    amount: number | bigint | BN,
    finalLongToken: PublicKey,
    finalShortToken: PublicKey,
    {
        executionFee = 5_001,
        computeUnits = 400_000,
        storeProgram,
        longTokenSwapPath,
        shortTokenSwapPath,
    }: {
        executionFee?: number | bigint,
        computeUnits?: number,
        storeProgram?: StoreProgram,
        longTokenSwapPath?: PublicKey[],
        shortTokenSwapPath?: PublicKey[],
    }
) => {
    let withdrawal: PublicKey;
    try {
        const [signature, address] = await invokeCreateWithdrawalWithPayerAsSigner(
            exchangeProgram,
            {
                store,
                payer: user,
                marketToken,
                amount,
                finalLongToken,
                finalShortToken,
                options: {
                    tokenMap,
                    longTokenSwapPath,
                    shortTokenSwapPath,
                }
            }
        );
        console.log(`withdrawal of amount ${amount} created at ${signature}`);
        withdrawal = address;
    } catch (error) {
        console.log(error);
        throw error;
    } finally {
        if (storeProgram) {
            const afterExecution = await storeProgram.account.oracle.fetch(oracle);
            expect(afterExecution.primary.prices.length).equals(0);
        }
    }
    try {
        const signature = await executeWithdrawal(
            false,
            exchangeProgram.provider.connection,
            {
                authority: keeper,
                store,
                oracle,
                withdrawal,
                options: {
                    executionFee,
                }
            },
            {
                computeUnits,
            },
        );
        console.log(`withdrawal executed at ${signature}`);
    } catch (error) {
        console.log(error);
        throw error;
    } finally {
        if (storeProgram) {
            const afterExecution = await storeProgram.account.oracle.fetch(oracle);
            expect(afterExecution.primary.prices.length).equals(0);
        }
    }
};

export const swap = async (
    storeProgram: StoreProgram,
    exchangeProgram: ExchangeProgram,
    user: Signer,
    keeper: Signer,
    store: PublicKey,
    tokenMap: PublicKey,
    oracle: PublicKey,
    marketToken: PublicKey,
    swapOutToken: PublicKey,
    initialSwapInToken: PublicKey,
    initialSwapInTokenAmount: number | bigint,
    swapPath: PublicKey[],
    {
        executionFee = 5_001,
        computeUnits = 400_000,
    }: {
        executionFee?: number,
        computeUnits?: number,
    }
) => {
    let order: PublicKey;
    const recentTimestamp = toInteger(Date.now() / 1000);
    try {
        const [signature, address] = await invokeCreateSwapOrderWithPayerAsSigner(exchangeProgram, {
            store,
            payer: user,
            marketToken,
            swapOutToken,
            initialSwapInToken,
            initialSwapInTokenAmount,
            swapPath,
            options: {
                tokenMap,
                storeProgram,
            }
        }, {
            computeUnits,
        });
        order = address;
        console.log(`swap order ${order} created at ${signature}`);
    } catch (error) {
        console.log(error);
        throw error;
    }
    try {
        const signature = await executeOrder(false, exchangeProgram.provider.connection, {
            authority: keeper,
            store,
            oracle,
            order,
            recentTimestamp,
            holding: exchangeProgram.provider.publicKey,
            options: {
                executionFee,
            }
        }, {
            computeUnits: 400_000,
        });
        console.log(`swap order ${order} executed at ${signature}`);
    } catch (error) {
        console.log(error);
        throw error;
    }
};
