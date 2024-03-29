import { workspace, Program, BN } from "@coral-xyz/anchor";
import { Exchange } from "../target/types/exchange";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createMarketPDA, createMarketTokenMintPDA, createMarketVaultPDA, createRolesPDA, createWithdrawalPDA, dataStore } from "./data";
import { TOKEN_PROGRAM_ID, getAccount } from "@solana/spl-token";
import { BTC_TOKEN_MINT, SOL_TOKEN_MINT } from "./token";
import { toBN } from "./number";

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
