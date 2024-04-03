import { workspace, Program, BN } from "@coral-xyz/anchor";
import { Exchange } from "../target/types/exchange";
import { ComputeBudgetProgram, Connection, Keypair, PublicKey, Signer, Transaction } from "@solana/web3.js";
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

    return marketTokenMint;
};

export type MakeCreateDepositParams = {
    authority: PublicKey,
    store: PublicKey,
    payer: PublicKey,
    marketToken: PublicKey,
    toMarketTokenAccount: PublicKey,
    fromInitialLongTokenAccount?: PublicKey,
    fromInitialShortTokenAccount?: PublicKey,
    initialLongTokenAmount?: number | bigint,
    initialShortTokenAmount?: number | bigint,
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

const getDepositVault = async (connection: Connection, store: PublicKey, fromTokenAccount?: PublicKey, hint?: PublicKey) => {
    if (fromTokenAccount) {
        const token = hint ?? (await getAccount(connection, fromTokenAccount)).mint;
        return createMarketVaultPDA(store, token)[0];
    }
};

export const makeCreateDepositInstruction = async ({
    authority,
    store,
    payer,
    marketToken,
    toMarketTokenAccount,
    fromInitialLongTokenAccount,
    fromInitialShortTokenAccount,
    initialLongTokenAmount,
    initialShortTokenAmount,
    options,
}: MakeCreateDepositParams) => {
    const depositNonce = options?.nonce ?? Keypair.generate().publicKey.toBuffer();
    const [deposit] = createDepositPDA(store, payer, depositNonce);
    const longTokenDepositVault = await getDepositVault(exchange.provider.connection, store, fromInitialLongTokenAccount, options?.hints?.initialLongToken);
    const shortTokenDepositVault = await getDepositVault(exchange.provider.connection, store, fromInitialShortTokenAccount, options?.hints?.initialShortToken);
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
        authority,
        store,
        onlyController: createRolesPDA(store, authority)[0],
        dataStoreProgram: dataStore.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        market: createMarketPDA(store, marketToken)[0],
        tokenConfigMap: createTokenConfigMapPDA(store)[0],
        deposit,
        payer,
        receiver: toMarketTokenAccount,
        initialLongTokenAccount: fromInitialLongTokenAccount ?? null,
        initialShortTokenAccount: fromInitialShortTokenAccount ?? null,
        longTokenDepositVault: longTokenDepositVault ?? null,
        shortTokenDepositVault: shortTokenDepositVault ?? null,
    }).remainingAccounts([...longSwapPath, ...shortSwapPath].map(mint => {
        return {
            pubkey: createMarketPDA(store, mint)[0],
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
                fromInitialLongTokenAccount: PublicKey | null,
                fromInitialShortTokenAccount: PublicKey | null,
                initialLongToken: PublicKey | null,
                initialShortToken: PublicKey | null,
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
            fromInitialLongTokenAccount: deposit.fixed.senders.initialLongTokenAccount ?? null,
            fromInitialShortTokenAccount: deposit.fixed.senders.initialShortTokenAccount ?? null,
            initialLongToken: deposit.fixed.tokens.initialLongToken ?? null,
            initialShortToken: deposit.fixed.tokens.initialShortToken ?? null,
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
        longTokenDepositVault: initialLongToken ? createMarketVaultPDA(store, initialLongToken)[0] : null,
        shortTokenDepositVault: initialShortToken ? createMarketVaultPDA(store, initialShortToken)[0] : null,
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
                marketToken: PublicKey,
                toMarketTokenAccount: PublicKey,
                feeds: PublicKey[],
                longSwapPath: PublicKey[],
                shortSwapPath: PublicKey[],
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
    const { user, marketToken, toMarketTokenAccount, feeds, longSwapPath, shortSwapPath } = options?.hints?.deposit ?? await exchange.account.deposit.fetch(deposit).then(deposit => {
        return {
            user: deposit.fixed.senders.user,
            market: deposit.fixed.market,
            marketToken: deposit.fixed.tokens.marketToken,
            toMarketTokenAccount: deposit.fixed.receivers.receiver,
            feeds: deposit.dynamic.tokensWithFeed.feeds,
            longSwapPath: deposit.dynamic.swapParams.longTokenSwapPath,
            shortSwapPath: deposit.dynamic.swapParams.shortTokenSwapPath,
        }
    });
    const feedAccounts = feeds.map(feed => {
        return {
            pubkey: feed,
            isSigner: false,
            isWritable: false,
        }
    });
    const swapPathMints = [...longSwapPath, ...shortSwapPath].map(mint => {
        return {
            pubkey: mint,
            isSigner: false,
            isWritable: false,
        }
    });
    const swapPathMarkets = [...longSwapPath, ...shortSwapPath].map(mint => {
        return {
            pubkey: createMarketPDA(store, mint)[0],
            isSigner: false,
            isWritable: true,
        };
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
        market: createMarketPDA(store, marketToken)[0],
        marketTokenMint: marketToken,
        user,
        deposit,
        receiver: toMarketTokenAccount,
    }).remainingAccounts([...feedAccounts, ...swapPathMarkets, ...swapPathMints]).instruction();
};

export const invokeExecuteDeposit = makeInvoke(makeExecuteDepositInstruction, ["authority"]);

export type MakeCreateWithdrawalParams = {
    authority: PublicKey,
    store: PublicKey,
    payer: PublicKey,
    marketToken: PublicKey,
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
    }
};

export const makeCreateWithdrawalInstruction = async ({
    authority,
    store,
    payer,
    marketToken,
    amount,
    fromMarketTokenAccount,
    toLongTokenAccount,
    toShortTokenAccount,
    options,
}: MakeCreateWithdrawalParams) => {
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
        market: createMarketPDA(store, marketToken)[0],
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
        market: createMarketPDA(store, marketTokenMint)[0],
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
    let GMWsolWsolBtc: PublicKey;
    try {
        GMWsolWsolBtc = await createMarket(signer, dataStoreAddress, SOL_TOKEN_MINT, SOL_TOKEN_MINT, BTC_TOKEN_MINT);
        console.log(`New market has been created, mint: ${GMWsolWsolBtc}`);
    } catch (error) {
        console.warn("Failed to initialize market", error);
    }

    let GMWsolWsolUsdG: PublicKey;
    try {
        GMWsolWsolUsdG = await createMarket(signer, dataStoreAddress, SOL_TOKEN_MINT, SOL_TOKEN_MINT, usdGMint);
        console.log(`New market has been created, mint: ${GMWsolWsolUsdG}`);
    } catch (error) {
        console.warn("Failed to initialize market", error);
    }

    let GMFakeFakeUsdG: PublicKey;
    try {
        GMFakeFakeUsdG = await createMarket(signer, dataStoreAddress, fakeTokenMint, fakeTokenMint, usdGMint);
        console.log(`New market has been created, mint: ${GMFakeFakeUsdG}`);
    } catch (error) {
        console.warn("Failed to initialize market", error);
    }

    return {
        GMWsolWsolBtc,
        GMWsolWsolUsdG,
        GMFakeFakeUsdG,
    }
};
