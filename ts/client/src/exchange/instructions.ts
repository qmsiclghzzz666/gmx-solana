import { Keypair, PublicKey } from "@solana/web3.js";
import { findControllerPDA } from ".";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { toBN } from "../utils/number";
import { findDepositPDA, findMarketPDA, findMarketVaultPDA, findRolesPDA, findTokenConfigMapPDA } from "../store";
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
    const market = findMarketPDA(store, marketToken)[0];
    const fromInitialLongTokenAccount = getTokenAccount(payer, initialLongToken, options?.fromInitialLongTokenAccount);
    const fromInitialShortTokenAccount = getTokenAccount(payer, initialShortToken, options?.fromInitialShortTokenAccount);
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
            initialLongTokenAmount: toBN(initialLongTokenAmount ?? 0),
            initialShortTokenAmount: toBN(initialShortTokenAmount ?? 0),
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
        initialLongTokenAccount: fromInitialLongTokenAccount ?? null,
        initialShortTokenAccount: fromInitialShortTokenAccount ?? null,
        longTokenDepositVault: longTokenDepositVault ?? null,
        shortTokenDepositVault: shortTokenDepositVault ?? null,
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
export const invokeCreateDeposit = makeInvoke(makeCreateDepositInstruction, []);
