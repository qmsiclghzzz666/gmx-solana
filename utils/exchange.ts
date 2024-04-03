import { workspace, Program, BN } from "@coral-xyz/anchor";
import { Exchange } from "../target/types/exchange";
import { ComputeBudgetProgram, Keypair, PublicKey, Signer, Transaction } from "@solana/web3.js";
import { createDepositPDA, createMarketPDA, createMarketTokenMintPDA, createMarketVaultPDA, createRolesPDA, createTokenConfigMapPDA, createWithdrawalPDA, dataStore, getTokenConfig } from "./data";
import { TOKEN_PROGRAM_ID, getAccount } from "@solana/spl-token";
import { BTC_TOKEN_MINT, SOL_TOKEN_MINT } from "./token";
import { toBN } from "./number";
import { oracle, oracle as oracleProgram } from "./oracle";
import { CHAINLINK_ID } from "./external";
import { IxWithOutput, makeInvoke } from "./invoke";

export const exchange = workspace.Exchange as Program<Exchange>;

export const createMarket = async (
    signer: Keypair,
    dataStoreAddress: PublicKey,
    indexTokenMint: PublicKey,
    longTokenMint: PublicKey,
    shortTokenMint: PublicKey,
) => {
    const [marketTokenMint] = createMarketTokenMintPDA(dataStoreAddress, indexTokenMint, longTokenMint, shortTokenMint);
    const [roles] = createRolesPDA(dataStoreAddress, signer.publicKey);
    const [marketAddress] = createMarketPDA(dataStoreAddress, marketTokenMint);
    const [marketTokenVault] = createMarketVaultPDA(dataStoreAddress, marketTokenMint);

    await exchange.methods.createMarket(indexTokenMint).accounts({
        authority: signer.publicKey,
        onlyMarketKeeper: roles,
        dataStore: dataStoreAddress,
        market: marketAddress,
        marketTokenMint,
        longTokenMint,
        shortTokenMint,
        marketTokenVault,
        dataStoreProgram: dataStore.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([signer]).rpc();

    return marketAddress;
};

export type MakeCreateDepositParams = {
    authority: PublicKey,
    store: PublicKey,
    payer: PublicKey,
    market: PublicKey,
    toMarketTokenAccount: PublicKey,
    fromInitialLongTokenAccount: PublicKey,
    fromInitialShortTokenAccount: PublicKey,
    initialLongTokenAmount: number | bigint,
    initialShortTokenAmount: number | bigint,
    options?: {
        nonce?: Buffer,
        executionFee?: number | bigint,
        longTokenSwapPath?: PublicKey[],
        shortTokenSwapPath?: PublicKey[],
        minMarketToken?: number | bigint,
        shouldUnwrapNativeToken?: boolean,
        hints?: {
            initialLongToken?: PublicKey,
            initialShortToken?: PublicKey,
        },
    },
}

export const makeCreateDepositInstruction = async ({
    authority,
    store,
    payer,
    market,
    toMarketTokenAccount,
    fromInitialLongTokenAccount,
    fromInitialShortTokenAccount,
    initialLongTokenAmount,
    initialShortTokenAmount,
    options,
}: MakeCreateDepositParams) => {
    const depositNonce = options?.nonce ?? Keypair.generate().publicKey.toBuffer();
    const [deposit] = createDepositPDA(store, payer, depositNonce);
    const initialLongToken = options?.hints?.initialLongToken ?? (await getAccount(exchange.provider.connection, fromInitialLongTokenAccount)).mint;
    const initialShortToken = options?.hints?.initialShortToken ?? (await getAccount(exchange.provider.connection, fromInitialShortTokenAccount)).mint;
    const longSwapPath = options?.longTokenSwapPath ?? [];
    const shortSwapPath = options?.shortTokenSwapPath ?? [];
    let instruction = await exchange.methods.createDeposit(
        [...depositNonce],
        {
            uiFeeReceiver: Keypair.generate().publicKey,
            executionFee: toBN(options?.executionFee ?? 0),
            longTokenSwapLength: longSwapPath.length,
            shortTokenSwapLength: shortSwapPath.length,
            initialLongTokenAmount: toBN(initialLongTokenAmount),
            initialShortTokenAmount: toBN(initialShortTokenAmount),
            minMarketToken: toBN(options?.minMarketToken ?? 0),
            shouldUnwrapNativeToken: options?.shouldUnwrapNativeToken ?? false,
        }
    ).accounts({
        authority,
        store,
        onlyController: createRolesPDA(store, authority)[0],
        dataStoreProgram: dataStore.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        market,
        tokenConfigMap: createTokenConfigMapPDA(store)[0],
        deposit,
        payer,
        receiver: toMarketTokenAccount,
        initialLongTokenAccount: fromInitialLongTokenAccount,
        initialShortTokenAccount: fromInitialShortTokenAccount,
        longTokenDepositVault: createMarketVaultPDA(store, initialLongToken)[0],
        shortTokenDepositVault: createMarketVaultPDA(store, initialShortToken)[0],
    }).remainingAccounts([...longSwapPath, ...shortSwapPath].map(pubkey => {
        return {
            pubkey,
            isSigner: false,
            isWritable: false,
        }
    })).instruction();

    return [instruction, deposit] as IxWithOutput<PublicKey>;
}

export const invokeCreateDeposit = makeInvoke(makeCreateDepositInstruction, ["payer", "authority"]);

export type MakeCancelDepositParams = {
    authority: PublicKey,
    store: PublicKey,
    deposit: PublicKey,
    options?: {
        executionFee?: number | bigint,
        hints?: {
            deposit?: {
                user: PublicKey,
                fromInitialLongTokenAccount: PublicKey,
                fromInitialShortTokenAccount: PublicKey,
                initialLongToken: PublicKey,
                initialShortToken: PublicKey,
            }
        }
    }
};

export const makeCancelDepositInstruction = async ({
    authority,
    store,
    deposit,
    options,
}: MakeCancelDepositParams) => {
    const {
        user,
        fromInitialLongTokenAccount,
        fromInitialShortTokenAccount,
        initialLongToken,
        initialShortToken,
    } = options?.hints?.deposit ?? await dataStore.account.deposit.fetch(deposit).then(deposit => {
        return {
            user: deposit.fixed.senders.user,
            fromInitialLongTokenAccount: deposit.fixed.senders.initialLongTokenAccount,
            fromInitialShortTokenAccount: deposit.fixed.senders.initialShortTokenAccount,
            initialLongToken: deposit.fixed.tokens.initialLongToken,
            initialShortToken: deposit.fixed.tokens.initialShortToken,
        }
    });

    return await exchange.methods.cancelDeposit(toBN(options?.executionFee ?? 0)).accounts({
        authority,
        store,
        onlyController: createRolesPDA(store, authority)[0],
        dataStoreProgram: dataStore.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        user,
        deposit,
        initialLongToken: fromInitialLongTokenAccount,
        initialShortToken: fromInitialShortTokenAccount,
        longTokenDepositVault: createMarketVaultPDA(store, initialLongToken)[0],
        shortTokenDepositVault: createMarketVaultPDA(store, initialShortToken)[0],
    }).instruction();
};

export const invokeCancelDeposit = makeInvoke(makeCancelDepositInstruction, ["authority"]);

export type MakeExecuteDepositParams = {
    authority: PublicKey,
    store: PublicKey,
    oracle: PublicKey,
    deposit: PublicKey,
    options?: {
        executionFee?: number | bigint,
        hints?: {
            deposit?: {
                user: PublicKey,
                market: PublicKey,
                marketToken: PublicKey,
                toMarketTokenAccount: PublicKey,
                feeds: PublicKey[],
            },
        }
    }
};

export const makeExecuteDepositInstruction = async ({
    authority,
    store,
    oracle,
    deposit,
    options
}: MakeExecuteDepositParams,
) => {
    const { user, market, marketToken, toMarketTokenAccount, feeds } = options?.hints?.deposit ?? await exchange.account.deposit.fetch(deposit).then(deposit => {
        return {
            user: deposit.fixed.senders.user,
            market: deposit.fixed.market,
            marketToken: deposit.fixed.tokens.marketToken,
            toMarketTokenAccount: deposit.fixed.receivers.receiver,
            feeds: deposit.dynamic.tokensWithFeed.feeds,
        }
    });
    return await exchange.methods.executeDeposit(toBN(options?.executionFee ?? 0)).accounts({
        authority,
        store,
        onlyOrderKeeper: createRolesPDA(store, authority)[0],
        dataStoreProgram: dataStore.programId,
        oracleProgram: oracleProgram.programId,
        oracle,
        chainlinkProgram: CHAINLINK_ID,
        tokenConfigMap: createTokenConfigMapPDA(store)[0],
        market,
        marketTokenMint: marketToken,
        user,
        deposit,
        receiver: toMarketTokenAccount,
    }).remainingAccounts(feeds.map(feed => {
        return {
            pubkey: feed,
            isSigner: false,
            isWritable: false,
        }
    })).instruction();
};

export const invokeExecuteDeposit = makeInvoke(makeExecuteDepositInstruction, ["authority"]);

export type MakeCreateWithdrawalParams = {
    authority: PublicKey,
    store: PublicKey,
    payer: PublicKey,
    market: PublicKey,
    amount: number | bigint,
    fromMarketTokenAccount: PublicKey,
    toLongTokenAccount: PublicKey,
    toShortTokenAccount: PublicKey,
    options?: {
        nonce?: Buffer,
        executionFee?: number | bigint,
        minLongTokenAmount?: number | bigint,
        minShortTokenAmount?: number | bigint,
        longTokenSwapPath?: PublicKey[],
        shortTokenSwapPath?: PublicKey[],
        shouldUnwrapNativeToken?: boolean,
        hints?: {
            marketToken?: PublicKey,
        },
    }
};

export const makeCreateWithdrawalInstruction = async ({
    authority,
    store,
    payer,
    market,
    amount,
    fromMarketTokenAccount,
    toLongTokenAccount,
    toShortTokenAccount,
    options,
}: MakeCreateWithdrawalParams) => {
    const marketToken: PublicKey = options?.hints?.marketToken ?? await dataStore.methods.getMarketTokenMint().accounts({
        market,
    }).view();
    const withdrawalNonce = options?.nonce ?? Keypair.generate().publicKey.toBuffer();
    const [withdrawalAddress] = createWithdrawalPDA(store, payer, withdrawalNonce);
    const instruction = await exchange.methods.createWithdrawal([...withdrawalNonce], {
        marketTokenAmount: toBN(amount),
        executionFee: toBN(options?.executionFee ?? 0),
        uiFeeReceiver: PublicKey.default,
        tokens: {
            minLongTokenAmount: toBN(options?.minLongTokenAmount ?? 0),
            minShortTokenAmount: toBN(options?.minShortTokenAmount ?? 0),
            shouldUnwrapNativeToken: options?.shouldUnwrapNativeToken ?? false
        },
        swaps: {
            longTokenSwapPath: options?.longTokenSwapPath ?? [],
            shortTokenSwapPath: options?.shortTokenSwapPath ?? [],
        }
    }).accounts({
        authority,
        store,
        onlyController: createRolesPDA(store, authority)[0],
        dataStoreProgram: dataStore.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenConfigMap: createTokenConfigMapPDA(store)[0],
        market,
        withdrawal: withdrawalAddress,
        payer,
        marketTokenAccount: fromMarketTokenAccount,
        marketTokenWithdrawalVault: createMarketVaultPDA(store, marketToken)[0],
        finalLongTokenReceiver: toLongTokenAccount,
        finalShortTokenReceiver: toShortTokenAccount,
    }).instruction();

    return [instruction, withdrawalAddress] as IxWithOutput<PublicKey>;
};

export const invokeCreateWithdrawal = makeInvoke(makeCreateWithdrawalInstruction, ["payer", "authority"]);

export type MakeCancelWithdrawalParams = {
    authority: PublicKey,
    store: PublicKey,
    withdrawal: PublicKey,
    options?: {
        executionFee?: number | bigint,
        hints?: {
            withdrawal?: {
                user: PublicKey,
                marketToken: PublicKey,
                toMarketTokenAccount: PublicKey,
            }
        }
    },
};

export const makeCancelWithdrawalInstruction = async ({
    authority,
    store,
    withdrawal,
    options,
}: MakeCancelWithdrawalParams) => {
    const { marketToken, user, toMarketTokenAccount } = options?.hints?.withdrawal ?? await dataStore.account.withdrawal.fetch(withdrawal).then(withdrawal => {
        return {
            user: withdrawal.fixed.user,
            marketToken: withdrawal.fixed.tokens.marketToken,
            toMarketTokenAccount: withdrawal.fixed.marketTokenAccount,
        }
    });
    return await exchange.methods.cancelWithdrawal(toBN(options?.executionFee ?? 0)).accounts({
        authority,
        store,
        onlyController: createRolesPDA(store, authority)[0],
        dataStoreProgram: dataStore.programId,
        withdrawal,
        user,
        marketToken: toMarketTokenAccount,
        marketTokenWithdrawalVault: createMarketVaultPDA(store, marketToken)[0],
        tokenProgram: TOKEN_PROGRAM_ID,
    }).instruction();
};

export const invokeCancelWithdrawal = makeInvoke(makeCancelWithdrawalInstruction, ["authority"]);

export type MakeExecuteWithdrawalParams = {
    authority: PublicKey,
    store: PublicKey,
    oracle: PublicKey,
    withdrawal: PublicKey,
    options?: {
        executionFee?: number | bigint,
        hints?: {
            withdrawal?: {
                user: PublicKey,
                market: PublicKey,
                marketTokenMint: PublicKey,
                finalLongTokenReceiver: PublicKey,
                finalShortTokenReceiver: PublicKey,
                finalLongTokenMint: PublicKey,
                finalShortTokenMint: PublicKey,
                feeds: PublicKey[],
            }
        }
    },
};

export const makeExecuteWithdrawalInstruction = async ({
    authority,
    store,
    oracle,
    withdrawal,
    options,
}: MakeExecuteWithdrawalParams) => {
    const {
        user,
        market,
        marketTokenMint,
        finalLongTokenReceiver,
        finalShortTokenReceiver,
        finalLongTokenMint,
        finalShortTokenMint,
        feeds,
    } = options?.hints?.withdrawal ?? (
        await dataStore.account.withdrawal.fetch(withdrawal).then(withdrawal => {
            return {
                user: withdrawal.fixed.user,
                market: withdrawal.fixed.market,
                marketTokenMint: withdrawal.fixed.tokens.marketToken,
                finalLongTokenMint: withdrawal.fixed.tokens.finalLongToken,
                finalShortTokenMint: withdrawal.fixed.tokens.finalShortToken,
                finalLongTokenReceiver: withdrawal.fixed.receivers.finalLongTokenReceiver,
                finalShortTokenReceiver: withdrawal.fixed.receivers.finalShortTokenReceiver,
                feeds: withdrawal.dynamic.tokensWithFeed.feeds,
            }
        }));
    return await exchange.methods.executeWithdrawal(toBN(options?.executionFee ?? 0)).accounts({
        authority,
        store,
        onlyOrderKeeper: createRolesPDA(store, authority)[0],
        dataStoreProgram: dataStore.programId,
        oracleProgram: oracleProgram.programId,
        chainlinkProgram: CHAINLINK_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        oracle,
        tokenConfigMap: createTokenConfigMapPDA(store)[0],
        withdrawal,
        user,
        market,
        marketTokenMint,
        marketTokenWithdrawalVault: createMarketVaultPDA(store, marketTokenMint)[0],
        finalLongTokenReceiver,
        finalShortTokenReceiver,
        finalLongTokenVault: createMarketVaultPDA(store, finalLongTokenMint)[0],
        finalShortTokenVault: createMarketVaultPDA(store, finalShortTokenMint)[0],
    }).remainingAccounts(feeds.map(feed => {
        return {
            pubkey: feed,
            isSigner: false,
            isWritable: false,
        }
    })).instruction();
};

export const invokeExecuteWithdrawal = makeInvoke(makeExecuteWithdrawalInstruction, ["authority"]);

export const initializeMarkets = async (signer: Keypair, dataStoreAddress: PublicKey, fakeTokenMint: PublicKey, usdGMint: PublicKey) => {
    let marketWsolWsolBtc: PublicKey;
    try {
        marketWsolWsolBtc = await createMarket(signer, dataStoreAddress, SOL_TOKEN_MINT, SOL_TOKEN_MINT, BTC_TOKEN_MINT);
        console.log(`New market has been created: ${marketWsolWsolBtc}`);
    } catch (error) {
        console.warn("Failed to initialize market", error);
    }

    let marketWsolWsolUsdG: PublicKey;
    try {
        marketWsolWsolUsdG = await createMarket(signer, dataStoreAddress, SOL_TOKEN_MINT, SOL_TOKEN_MINT, usdGMint);
        console.log(`New market has been created: ${marketWsolWsolUsdG}`);
    } catch (error) {
        console.warn("Failed to initialize market", error);
    }

    let marketFakeFakeUsdG: PublicKey;
    try {
        marketFakeFakeUsdG = await createMarket(signer, dataStoreAddress, fakeTokenMint, fakeTokenMint, usdGMint);
        console.log(`New market has been created: ${marketFakeFakeUsdG}`);
    } catch (error) {
        console.warn("Failed to initialize market", error);
    }

    return {
        marketWsolWsolBtc,
        marketWsolWsolUsdG,
        marketFakeFakeUsdG,
    }
};
