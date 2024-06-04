import { Keypair, PublicKey } from "@solana/web3.js";
import { findControllerPDA } from ".";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { toBN } from "../utils/number";
import { findDepositPDA, findMarketPDA, findMarketVaultPDA, findOrderPDA, findPositionPDA, findRolesPDA, findTokenConfigMapPDA, findWithdrawalPDA } from "../store";
import { IxWithOutput, makeInvoke } from "../utils/invoke";
import { DataStoreProgram, ExchangeProgram } from "../program";
import { BN } from "@coral-xyz/anchor";
import { getPositionSide } from "./utils";
import { first } from "lodash";

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
        initialLongTokenVault: fromInitialLongTokenAccount ? longTokenDepositVault : null,
        initialShortTokenVault: fromInitialShortTokenAccount ? shortTokenDepositVault : null,
        initialLongMarket: fromInitialLongTokenAccount ? findMarketPDA(store, first(longSwapPath) ?? marketToken)[0] : null,
        initialShortMarket: fromInitialShortTokenAccount ? findMarketPDA(store, first(shortSwapPath) ?? marketToken)[0] : null,
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

export type MakeCreateDecreaseOrderParams = {
    store: PublicKey,
    payer: PublicKey,
    position: PublicKey,
    initialCollateralDeltaAmount?: number | bigint,
    sizeDeltaUsd?: number | bigint,
    options: {
        nonce?: Buffer,
        executionFee?: number | bigint,
        swapPath?: PublicKey[],
        minOutputAmount?: number | bigint,
        acceptablePrice?: number | bigint,
        finalOutputToken?: PublicKey,
        finalOutputTokenAccount?: PublicKey,
        secondaryOutputTokenAccount?: PublicKey,
        longTokenAccount?: PublicKey,
        shortTokenAccount?: PublicKey,
        hint?: {
            market: {
                marketToken: PublicKey,
                longToken: PublicKey,
                shortToken: PublicKey,
            },
            collateralToken: PublicKey,
            isLong: boolean,
        },
        dataStore?: DataStoreProgram,
    }
};

export const makeCreateDecreaseOrderInstruction = async (
    exchange: ExchangeProgram,
    {
        store,
        payer,
        position,
        initialCollateralDeltaAmount,
        sizeDeltaUsd,
        options,
    }: MakeCreateDecreaseOrderParams
) => {
    let pnlToken: PublicKey;
    let collateralToken: PublicKey;
    let market: PublicKey;
    let isLong: boolean;
    let longToken: PublicKey;
    let shortToken: PublicKey;
    if (options.hint) {
        const { marketToken, ...rest } = options.hint.market;
        ({ longToken, shortToken } = rest);
        isLong = options.hint.isLong;
        collateralToken = options.hint.collateralToken;
        [market] = findMarketPDA(store, marketToken);
        pnlToken = isLong ? longToken : shortToken;
    } else if (options.dataStore) {
        const program = options.dataStore;
        const { kind, collateralToken: fetchedCollateralToken, marketToken } = await program.account.position.fetch(position);
        isLong = getPositionSide(kind)! === "long";
        collateralToken = fetchedCollateralToken;
        [market] = findMarketPDA(store, marketToken);
        const { meta: { longTokenMint, shortTokenMint } } = await program.account.market.fetch(market);
        longToken = longTokenMint;
        shortToken = shortTokenMint;
        pnlToken = isLong ? longTokenMint : shortTokenMint;
    } else {
        throw Error("Must provide either `hints` or `dataStore` program");
    }

    const swapPath = options?.swapPath ?? [];
    const [authority] = findControllerPDA(store);
    const [onlyController] = findRolesPDA(store, authority);
    const nonce = options?.nonce ?? Keypair.generate().publicKey.toBuffer();
    const [order] = findOrderPDA(store, payer, nonce);
    const acceptablePrice = options?.acceptablePrice;
    const finalOutputToken = options?.finalOutputToken ?? collateralToken;
    const finalOutputTokenAccount = getTokenAccount(payer, finalOutputToken, options?.finalOutputTokenAccount);
    const secondaryOutputTokenAccount = getTokenAccount(payer, pnlToken, options?.secondaryOutputTokenAccount);
    const longTokenAccount = getTokenAccount(payer, longToken, options?.longTokenAccount);
    const shortTokenAccount = getTokenAccount(payer, shortToken, options?.shortTokenAccount);

    const instruction = await exchange.methods.createOrder(
        [...nonce],
        {
            order: {
                kind: { "marketDecrease": {} },
                minOutputAmount: toBN(options?.minOutputAmount ?? 0),
                sizeDeltaUsd: toBN(sizeDeltaUsd ?? 0),
                initialCollateralDeltaAmount: toBN(initialCollateralDeltaAmount ?? 0),
                acceptablePrice: acceptablePrice ? toBN(acceptablePrice) : null,
                isLong,
            },
            outputToken: collateralToken,
            uiFeeReceiver: PublicKey.default,
            executionFee: toBN(options?.executionFee ?? 0),
            swapLength: swapPath.length,
        }).accounts({
            store,
            onlyController,
            payer,
            order,
            position,
            tokenConfigMap: findTokenConfigMapPDA(store)[0],
            market,
            initialCollateralTokenAccount: null,
            finalOutputTokenAccount,
            secondaryOutputTokenAccount,
            initialCollateralTokenVault: null,
            longTokenAccount,
            shortTokenAccount,
        }).remainingAccounts(swapPath.map(mint => {
            return {
                pubkey: findMarketPDA(store, mint)[0],
                isSigner: false,
                isWritable: false,
            }
        })).instruction();
    return [instruction, order] as IxWithOutput<PublicKey>;
};

export const invokeCreateDecreaseOrderWithPayerAsSigner = makeInvoke(makeCreateDecreaseOrderInstruction, ["payer"]);
export const invokeCreateDecreaseOrder = makeInvoke(makeCreateDecreaseOrderInstruction, [], true);

export type MakeCreateIncreaseOrderParams = {
    store: PublicKey,
    payer: PublicKey,
    marketToken: PublicKey,
    collateralToken: PublicKey,
    isLong: boolean,
    initialCollateralDeltaAmount?: number | bigint,
    sizeDeltaUsd?: number | bigint,
    options: {
        nonce?: Buffer,
        executionFee?: number | bigint,
        swapPath?: PublicKey[],
        minOutputAmount?: number | bigint,
        acceptablePrice?: number | bigint,
        initialCollateralToken?: PublicKey,
        initialCollateralTokenAccount?: PublicKey,
        longTokenAccount?: PublicKey,
        shortTokenAccount?: PublicKey,
        hint?: {
            longToken: PublicKey,
            shortToken: PublicKey,
        },
        dataStore?: DataStoreProgram,
    }
};

export const makeCreateIncreaseOrderInstruction = async (
    exchange: ExchangeProgram,
    {
        store,
        payer,
        marketToken,
        collateralToken,
        isLong,
        initialCollateralDeltaAmount,
        sizeDeltaUsd,
        options,
    }: MakeCreateIncreaseOrderParams
) => {
    const swapPath = options?.swapPath ?? [];
    const [authority] = findControllerPDA(store);
    const [onlyController] = findRolesPDA(store, authority);
    const nonce = options?.nonce ?? Keypair.generate().publicKey.toBuffer();
    const [order] = findOrderPDA(store, payer, nonce);
    const acceptablePrice = options?.acceptablePrice;
    const initialCollateralToken = options?.initialCollateralToken ?? collateralToken;
    const initialCollateralTokenAccount = getTokenAccount(payer, initialCollateralToken, options?.initialCollateralTokenAccount);
    const [market] = findMarketPDA(store, marketToken);
    const collateralTokens = options?.hint ? options.hint : options?.dataStore ? (await options.dataStore.account.market.fetch(market).then(market => {
        return {
            longToken: market.meta.longTokenMint,
            shortToken: market.meta.shortTokenMint,
        };
    })) : undefined;

    if (!collateralTokens) throw Error("Neither `hint` nor `dataStoreProgram` provided");
    const { longToken, shortToken } = collateralTokens;
    const longTokenAccount = getTokenAccount(payer, longToken, options?.longTokenAccount);
    const shortTokenAccount = getTokenAccount(payer, shortToken, options?.shortTokenAccount);

    const instruction = await exchange.methods.createOrder(
        [...nonce],
        {
            order: {
                kind: { "marketIncrease": {} },
                minOutputAmount: toBN(options?.minOutputAmount ?? 0),
                sizeDeltaUsd: toBN(sizeDeltaUsd ?? 0),
                initialCollateralDeltaAmount: toBN(initialCollateralDeltaAmount ?? 0),
                acceptablePrice: acceptablePrice ? toBN(acceptablePrice) : null,
                isLong,
            },
            outputToken: collateralToken,
            uiFeeReceiver: PublicKey.default,
            executionFee: toBN(options?.executionFee ?? 0),
            swapLength: swapPath.length,
        }).accounts({
            store,
            onlyController,
            payer,
            order,
            position: findPositionPDA(store, payer, marketToken, collateralToken, isLong)[0],
            tokenConfigMap: findTokenConfigMapPDA(store)[0],
            market: findMarketPDA(store, marketToken)[0],
            initialCollateralTokenAccount: initialCollateralTokenAccount,
            initialCollateralTokenVault: findMarketVaultPDA(store, initialCollateralToken)[0],
            finalOutputTokenAccount: null,
            secondaryOutputTokenAccount: null,
            longTokenAccount,
            shortTokenAccount,
        }).remainingAccounts(swapPath.map(mint => {
            return {
                pubkey: findMarketPDA(store, mint)[0],
                isSigner: false,
                isWritable: false,
            }
        })).instruction();
    return [instruction, order] as IxWithOutput<PublicKey>;
};

export const invokeCreateIncreaseOrderWithPayerAsSigner = makeInvoke(makeCreateIncreaseOrderInstruction, ["payer"]);
export const invokeCreateIncreaseOrder = makeInvoke(makeCreateIncreaseOrderInstruction, [], true);
