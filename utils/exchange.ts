import { workspace, Program, BN } from "@coral-xyz/anchor";
import { Exchange } from "../target/types/exchange";
import { ComputeBudgetProgram, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import { createMarketPDA, createMarketTokenMintPDA, createMarketVaultPDA, createRolesPDA, createTokenConfigPDA, createWithdrawalPDA, dataStore } from "./data";
import { TOKEN_PROGRAM_ID, getAccount } from "@solana/spl-token";
import { BTC_TOKEN_MINT, SOL_TOKEN_MINT } from "./token";
import { toBN } from "./number";
import { oracle as oracleProgram } from "./oracle";
import { CHAINLINK_ID } from "./external";

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

export interface WithdrawalOptions {
    nonce?: Buffer,
    executionFee?: number | bigint,
    minLongTokenAmount?: number | bigint,
    minShortTokenAmount?: number | bigint,
    longTokenSwapPath?: PublicKey[],
    shortTokenSwapPath?: PublicKey[],
    shouldUnwrapNativeToken?: boolean,
    hints?: WithdrawalHints,
    callback?: (string) => void,
}

export interface WithdrawalHints {
    marketToken?: PublicKey,
}

export const createWithdrawal = async (
    authority: Keypair,
    store: PublicKey,
    payer: Keypair,
    market: PublicKey,
    amount: number | bigint,
    fromMarketTokenAccount: PublicKey,
    toLongTokenAccount: PublicKey,
    toShortTokenAccount: PublicKey,
    options: WithdrawalOptions = {
        executionFee: 0,
        minLongTokenAmount: 0,
        minShortTokenAmount: 0,
        longTokenSwapPath: [],
        shortTokenSwapPath: [],
        shouldUnwrapNativeToken: false,
        hints: {},
    }
) => {
    const marketToken: PublicKey = options.hints?.marketToken ?? await dataStore.methods.getMarketTokenMint().accounts({
        market,
    }).view();
    const withdrawalNonce = options.nonce ?? Keypair.generate().publicKey.toBuffer();
    const [withdrawalAddress] = createWithdrawalPDA(store, payer.publicKey, withdrawalNonce);
    await exchange.methods.createWithdrawal([...withdrawalNonce], {
        marketTokenAmount: toBN(amount),
        executionFee: toBN(options.executionFee ?? 0),
        uiFeeReceiver: PublicKey.default,
        tokens: {
            minLongTokenAmount: toBN(options.minLongTokenAmount ?? 0),
            minShortTokenAmount: toBN(options.minShortTokenAmount ?? 0),
            longTokenSwapPath: options.longTokenSwapPath ?? [],
            shortTokenSwapPath: options.shortTokenSwapPath ?? [],
            shouldUnwrapNativeToken: options.shouldUnwrapNativeToken ?? false
        }
    }).accounts({
        authority: authority.publicKey,
        store,
        onlyController: createRolesPDA(store, authority.publicKey)[0],
        dataStoreProgram: dataStore.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        market,
        withdrawal: withdrawalAddress,
        payer: payer.publicKey,
        marketToken: fromMarketTokenAccount,
        marketTokenWithdrawalVault: createMarketVaultPDA(store, marketToken)[0],
        finalLongTokenReceiver: toLongTokenAccount,
        finalShortTokenReceiver: toShortTokenAccount,
    }).signers([authority, payer]).rpc().then(options.callback);

    return withdrawalAddress;
};

export interface CancelWithdrawalOptions {
    executionFee?: number | bigint,
    callback?: (string) => void,
    hints?: {
        marketToken?: PublicKey,
    }
};

export const cancelWithdrawal = async (
    authority: Keypair,
    store: PublicKey,
    user: PublicKey,
    withdrawal: PublicKey,
    toMarketTokenAccount: PublicKey,
    options: CancelWithdrawalOptions = {},
) => {
    const marketToken = options.hints?.marketToken ?? (await dataStore.account.withdrawal.fetch(withdrawal)).tokens.marketToken;
    await exchange.methods.cancelWithdrawal(toBN(options.executionFee ?? 0)).accounts({
        authority: authority.publicKey,
        store,
        onlyController: createRolesPDA(store, authority.publicKey)[0],
        dataStoreProgram: dataStore.programId,
        withdrawal,
        user,
        marketToken: toMarketTokenAccount,
        marketTokenWithdrawalVault: createMarketVaultPDA(store, marketToken)[0],
        tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([authority]).rpc().then(options.callback);
};

export interface ExecuteWithdrawalOptions {
    executionFee?: number | bigint,
    callback?: (string) => void,
    hints?: {
        market?: {
            address: PublicKey,
            mint: PublicKey,
        }
    }
};

export const executeWithdrawal = async (
    authority: Keypair,
    store: PublicKey,
    oracle: PublicKey,
    user: PublicKey,
    withdrawal: PublicKey,
    options: ExecuteWithdrawalOptions = {},
) => {
    const { address: market, mint: marketTokenMint } = options.hints?.market ?? (
        await dataStore.account.withdrawal.fetch(withdrawal).then(withdrawal => {
            return {
                address: withdrawal.market,
                mint: withdrawal.tokens.marketToken,
            }
        }));
    const marketMeta = await dataStore.methods.getMarketMeta().accounts({ market }).view();
    const longTokenConfig = createTokenConfigPDA(store, marketMeta.longTokenMint.toBase58())[0];
    const shortTokenConfig = createTokenConfigPDA(store, marketMeta.shortTokenMint.toBase58())[0];
    const longTokenFeed = (await dataStore.account.tokenConfig.fetch(longTokenConfig)).priceFeed;
    const shortTokenFeed = (await dataStore.account.tokenConfig.fetch(shortTokenConfig)).priceFeed;
    let ix = await exchange.methods.executeWithdrawal(toBN(options.executionFee ?? 0)).accounts({
        authority: authority.publicKey,
        store,
        onlyOrderKeeper: createRolesPDA(store, authority.publicKey)[0],
        dataStoreProgram: dataStore.programId,
        oracleProgram: oracleProgram.programId,
        chainlinkProgram: CHAINLINK_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        oracle,
        withdrawal,
        user,
        market,
        marketTokenMint,
    }).remainingAccounts([
        {
            pubkey: longTokenConfig,
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: longTokenFeed,
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: shortTokenConfig,
            isSigner: false,
            isWritable: false,
        },
        {
            pubkey: shortTokenFeed,
            isSigner: false,
            isWritable: false,
        }
    ]).instruction();
    const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
        units: 400_000
    });
    const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({
        microLamports: 1,
    });
    const tx = new Transaction().add(modifyComputeUnits).add(addPriorityFee).add(ix);
    await exchange.provider.sendAndConfirm(tx, [authority]).then(options.callback);
};

export const initializeMarkets = async (signer: Keypair, dataStoreAddress: PublicKey, fakeTokenMint: PublicKey, usdGMint: PublicKey) => {
    let marketSolSolBtc: PublicKey;
    try {
        marketSolSolBtc = await createMarket(signer, dataStoreAddress, SOL_TOKEN_MINT, SOL_TOKEN_MINT, BTC_TOKEN_MINT);
        console.log(`New market has been created: ${marketSolSolBtc}`);
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
        marketSolSolBtc,
        marketFakeFakeUsdG,
    }
};
