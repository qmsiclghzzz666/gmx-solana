import { Keypair, PublicKey } from "@solana/web3.js";
import { findControllerPDA } from ".";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { toBN } from "../utils/number";
import { findDepositPDA, findMarketPDA, findMarketVaultPDA, findRolesPDA, findTokenConfigMapPDA, findWithdrawalPDA } from "../store";
import { IxWithOutput, makeInvoke } from "../utils/invoke";
import { ExchangeProgram } from "../program";
import { BN } from "@coral-xyz/anchor";

export type MakeCreateDepositParams = {
    store: PublicKey,
    payer: PublicKey,
    marketToken: PublicKey,
    initialLongToken: PublicKey,
    initialShortToken: PublicKey,
    initialLongTokenAmount?: number | bigint | BN,
    initialShortTokenAmount?: number | bigint | BN,
    options?: {
        nonce?: Buffer,
        executionFee?: number | bigint,
        longTokenSwapPath?: PublicKey[],
        shortTokenSwapPath?: PublicKey[],
        minMarketToken?: number | bigint,
        shouldUnwrapNativeToken?: boolean,
        fromInitialLongTokenAccount?: PublicKey,
        fromInitialShortTokenAccount?: PublicKey,
        toMarketTokenAccount?: PublicKey,
    },
}

const getTokenAccount = (payer: PublicKey, token: PublicKey, account?: PublicKey) => {
    return account ? account : getAssociatedTokenAddressSync(token, payer);
}

export const makeCreateDepositInstruction = async (
    exchange: ExchangeProgram,
    {
        store,
        payer,
        marketToken,
        initialLongToken,
        initialShortToken,
        initialLongTokenAmount,
        initialShortTokenAmount,
        options,
    }: MakeCreateDepositParams) => {
    const initialLongTokenAmountBN = toBN(initialLongTokenAmount ?? 0);
    const initialShortTokenAmountBN = toBN(initialShortTokenAmount ?? 0);
    const market = findMarketPDA(store, marketToken)[0];
    const fromInitialLongTokenAccount = initialLongTokenAmountBN.isZero() ? null : getTokenAccount(payer, initialLongToken, options?.fromInitialLongTokenAccount);
    const fromInitialShortTokenAccount = initialShortTokenAmountBN.isZero() ? null : getTokenAccount(payer, initialShortToken, options?.fromInitialShortTokenAccount);
    const toMarketTokenAccount = getTokenAccount(payer, marketToken, options?.toMarketTokenAccount);
    const [authority] = findControllerPDA(store);
    const depositNonce = options?.nonce ?? Keypair.generate().publicKey.toBuffer();
    const [deposit] = findDepositPDA(store, payer, depositNonce);
    const [longTokenDepositVault] = findMarketVaultPDA(store, initialLongToken);
    const [shortTokenDepositVault] = findMarketVaultPDA(store, initialShortToken);
    const longSwapPath = options?.longTokenSwapPath ?? [];
    const shortSwapPath = options?.shortTokenSwapPath ?? [];
    let instruction = await exchange.methods.createDeposit(
        [...depositNonce],
        {
            uiFeeReceiver: Keypair.generate().publicKey,
            executionFee: toBN(options?.executionFee ?? 0),
            longTokenSwapLength: longSwapPath.length,
            shortTokenSwapLength: shortSwapPath.length,
            initialLongTokenAmount: initialLongTokenAmountBN,
            initialShortTokenAmount: initialShortTokenAmountBN,
            minMarketToken: toBN(options?.minMarketToken ?? 0),
            shouldUnwrapNativeToken: options?.shouldUnwrapNativeToken ?? false,
        }
    ).accounts({
        store,
        onlyController: findRolesPDA(store, authority)[0],
        market,
        tokenConfigMap: findTokenConfigMapPDA(store)[0],
        deposit,
        payer,
        receiver: toMarketTokenAccount,
        initialLongTokenAccount: fromInitialLongTokenAccount,
        initialShortTokenAccount: fromInitialShortTokenAccount,
        longTokenDepositVault: fromInitialLongTokenAccount ? longTokenDepositVault : null,
        shortTokenDepositVault: fromInitialShortTokenAccount ? shortTokenDepositVault : null,
    }).remainingAccounts([...longSwapPath, ...shortSwapPath].map(mint => {
        return {
            pubkey: findMarketPDA(store, mint)[0],
            isSigner: false,
            isWritable: false,
        }
    })).instruction();

    return [instruction, deposit] as IxWithOutput<PublicKey>;
}

export const invokeCreateDepositWithPayerAsSigner = makeInvoke(makeCreateDepositInstruction, ["payer"]);
export const invokeCreateDeposit = makeInvoke(makeCreateDepositInstruction, [], true);

export type MakeCreateWithdrawalParams = {
    store: PublicKey,
    payer: PublicKey,
    marketToken: PublicKey,
    amount: number | bigint | BN,
    finalLongToken: PublicKey,
    finalShortToken: PublicKey,
    options?: {
        nonce?: Buffer,
        executionFee?: number | bigint,
        minLongTokenAmount?: number | bigint,
        minShortTokenAmount?: number | bigint,
        fromMarketTokenAccount?: PublicKey,
        toLongTokenAccount?: PublicKey,
        toShortTokenAccount?: PublicKey,
        longTokenSwapPath?: PublicKey[],
        shortTokenSwapPath?: PublicKey[],
        shouldUnwrapNativeToken?: boolean,
    }
};

export const makeCreateWithdrawalInstruction = async (
    exchange: ExchangeProgram,
    {
        store,
        payer,
        marketToken,
        amount,
        finalLongToken,
        finalShortToken,
        options,
    }: MakeCreateWithdrawalParams) => {
    const [authority] = findControllerPDA(store);
    const fromMarketTokenAccount = getTokenAccount(payer, marketToken, options?.fromMarketTokenAccount);
    const toLongTokenAccount = getTokenAccount(payer, finalLongToken, options?.toLongTokenAccount);
    const toShortTokenAccount = getTokenAccount(payer, finalShortToken, options?.toShortTokenAccount);
    const withdrawalNonce = options?.nonce ?? Keypair.generate().publicKey.toBuffer();
    const [withdrawalAddress] = findWithdrawalPDA(store, payer, withdrawalNonce);
    const longSwapPath = options?.longTokenSwapPath ?? [];
    const shortSwapPath = options?.shortTokenSwapPath ?? [];
    const instruction = await exchange.methods.createWithdrawal([...withdrawalNonce], {
        marketTokenAmount: toBN(amount),
        executionFee: toBN(options?.executionFee ?? 0),
        uiFeeReceiver: PublicKey.default,
        tokens: {
            minLongTokenAmount: toBN(options?.minLongTokenAmount ?? 0),
            minShortTokenAmount: toBN(options?.minShortTokenAmount ?? 0),
            shouldUnwrapNativeToken: options?.shouldUnwrapNativeToken ?? false
        },
        longTokenSwapLength: longSwapPath.length,
        shortTokenSwapLength: shortSwapPath.length,
    }).accounts({
        store,
        onlyController: findRolesPDA(store, authority)[0],
        tokenConfigMap: findTokenConfigMapPDA(store)[0],
        market: findMarketPDA(store, marketToken)[0],
        withdrawal: withdrawalAddress,
        payer,
        marketTokenAccount: fromMarketTokenAccount,
        marketTokenWithdrawalVault: findMarketVaultPDA(store, marketToken)[0],
        finalLongTokenReceiver: toLongTokenAccount,
        finalShortTokenReceiver: toShortTokenAccount,
    }).remainingAccounts([...longSwapPath, ...shortSwapPath].map(token => {
        return {
            pubkey: findMarketPDA(store, token)[0],
            isSigner: false,
            isWritable: false,
        };
    })).instruction();

    return [instruction, withdrawalAddress] as IxWithOutput<PublicKey>;
};

export const invokeCreateWithdrawalWithPayerAsSigner = makeInvoke(makeCreateWithdrawalInstruction, ["payer"]);
export const invokeCreateWithdrawal = makeInvoke(makeCreateWithdrawalInstruction, [], true);
