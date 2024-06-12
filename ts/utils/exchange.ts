import { workspace, Program, utils, Wallet } from "@coral-xyz/anchor";
import { Exchange } from "../../target/types/exchange";
import { AccountMeta, Connection, Keypair, PublicKey } from "@solana/web3.js";
import { createMarketPDA, createMarketTokenMintPDA, createMarketVaultPDA, createRolesPDA, dataStore } from "./data";
import { getAccount } from "@solana/spl-token";
import { BTC_TOKEN_MINT, SOL_TOKEN_MINT } from "./token";
import { IxWithOutput, makeInvoke } from "./invoke";
import { DataStoreProgram, PriceProvider, findConfigPDA, findControllerPDA, findMarketPDA, findMarketVaultPDA, findRolesPDA, toBN } from "gmsol";
import { PYTH_ID } from "./external";
import { findKey, first, last, toInteger } from "lodash";
import { findPythPriceFeedPDA } from "gmsol";
import { PriceServiceConnection } from "@pythnetwork/price-service-client";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import { findClaimableAccountPDA, getTimeKey, invokeCloseEmptyClaimableAccount, invokeUseClaimableAccount } from "./data/token";
import { TIME_WINDOW } from "./data/constants";
import { makeInvoke as makeInvoke2 } from "gmsol";

export const exchange = workspace.Exchange as Program<Exchange>;

export const createControllerPDA = (store: PublicKey) => PublicKey.findProgramAddressSync([
    utils.bytes.utf8.encode("controller"),
    store.toBuffer(),
], exchange.programId);

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
        dataStore: dataStoreAddress,
        market: marketAddress,
        marketTokenMint,
        longTokenMint,
        shortTokenMint,
        marketTokenVault,
    }).signers([signer]).rpc();

    return marketTokenMint;
};

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
                initialLongMarket: PublicKey | null,
                initialShortMarket: PublicKey | null,
            }
        }
    }
};

export const makeUpdateMarketConfigInstruction = async (
    program: DataStoreProgram,
    {
        authority,
        store,
        marketToken,
        key,
        value,
    }: {
        authority: PublicKey,
        store: PublicKey,
        marketToken: PublicKey,
        key: string,
        value: bigint | number,
    }
) => {
    return await program.methods.updateMarketConfig(key, toBN(value)).accountsPartial({
        authority,
        store,
        market: findMarketPDA(store, marketToken)[0],
    }).instruction();
};

export const invokeUpdateMarketConfig = makeInvoke2(makeUpdateMarketConfigInstruction, ["authority"]);

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
        initialLongMarket,
        initialShortMarket,
    } = options?.hints?.deposit ?? await dataStore.account.deposit.fetch(deposit).then(deposit => {
        const marketToken = deposit.fixed.tokens.marketToken;
        const initialLongToken = deposit.fixed.tokens.initialLongToken ?? null;
        const initialShortToken = deposit.fixed.tokens.initialShortToken ?? null;
        return {
            user: deposit.fixed.senders.user,
            fromInitialLongTokenAccount: deposit.fixed.senders.initialLongTokenAccount ?? null,
            fromInitialShortTokenAccount: deposit.fixed.senders.initialShortTokenAccount ?? null,
            initialLongToken,
            initialShortToken,
            initialLongMarket: initialLongToken ? findMarketPDA(deposit.fixed.store, first(deposit.dynamic.swapParams.longTokenSwapPath) ?? marketToken)[0] : null,
            initialShortMarket: initialShortToken ? findMarketPDA(deposit.fixed.store, first(deposit.dynamic.swapParams.shortTokenSwapPath) ?? marketToken)[0] : null,
        }
    });

    return await exchange.methods.cancelDeposit(toBN(options?.executionFee ?? 0)).accounts({
        authority,
        store,
        user,
        deposit,
        initialLongToken: fromInitialLongTokenAccount,
        initialShortToken: fromInitialShortTokenAccount,
        longTokenDepositVault: initialLongToken ? createMarketVaultPDA(store, initialLongToken)[0] : null,
        shortTokenDepositVault: initialShortToken ? createMarketVaultPDA(store, initialShortToken)[0] : null,
        initialLongMarket,
        initialShortMarket,
    }).instruction();
};

export const invokeCancelDeposit = makeInvoke(makeCancelDepositInstruction, ["authority"]);

export interface DepositHint {
    user: PublicKey,
    marketToken: PublicKey,
    toMarketTokenAccount: PublicKey,
    feeds: PublicKey[],
    longSwapPath: PublicKey[],
    shortSwapPath: PublicKey[],
    providerMapper: (number) => number | undefined,
}

export type MakeExecuteDepositParams = {
    authority: PublicKey,
    store: PublicKey,
    oracle: PublicKey,
    deposit: PublicKey,
    options?: {
        executionFee?: number | bigint,
        priceProvider?: PublicKey,
        hints?: {
            deposit?: DepositHint,
        }
    }
};

const getSelectedProvider = (provider: number) => PriceProvider[findKey(PriceProvider, p => p === provider) as keyof typeof PriceProvider];

const getFeedAccountMeta = (provider: number, feed: PublicKey) => {
    const selectedProvider = getSelectedProvider(provider);
    let pubkey: PublicKey = feed;
    if (selectedProvider === PriceProvider.Pyth) {
        pubkey = findPythPriceFeedPDA(0, feed.toBuffer())[0];
    }
    return {
        pubkey,
        isSigner: false,
        isWritable: false,
    } satisfies AccountMeta as AccountMeta;
};

const makeProviderMapper = (providers: number[], lenghts: number[]) => {
    const ranges: Array<{ start: number, end: number, provider: number }> = [];
    let startIndex = 0;
    for (let i = 0; i < lenghts.length; i++) {
        let endIndex = startIndex + lenghts[i];
        ranges.push({ start: startIndex, end: endIndex, provider: providers[i] });
        startIndex = endIndex;
    }
    return (index: number) => {
        const range = ranges.find(range => index >= range.start && index < range.end);
        return range ? range.provider : undefined;
    }
};

const fetchDepositHint = async (dataStore: DataStoreProgram, deposit: PublicKey) => {
    return await dataStore.account.deposit.fetch(deposit).then(deposit => {
        return {
            user: deposit.fixed.senders.user,
            market: deposit.fixed.market,
            marketToken: deposit.fixed.tokens.marketToken,
            toMarketTokenAccount: deposit.fixed.receivers.receiver,
            feeds: deposit.dynamic.tokensWithFeed.feeds,
            longSwapPath: deposit.dynamic.swapParams.longTokenSwapPath,
            shortSwapPath: deposit.dynamic.swapParams.shortTokenSwapPath,
            providerMapper: makeProviderMapper(
                [...deposit.dynamic.tokensWithFeed.providers],
                deposit.dynamic.tokensWithFeed.nums,
            )
        }
    }) satisfies DepositHint as DepositHint;
};

export const makeExecuteDepositInstruction = async ({
    authority,
    store,
    oracle,
    deposit,
    options
}: MakeExecuteDepositParams,
) => {
    const {
        user,
        marketToken,
        toMarketTokenAccount,
        feeds,
        longSwapPath,
        shortSwapPath,
        providerMapper,
    } = options?.hints?.deposit ?? await fetchDepositHint(dataStore, deposit);
    const feedAccounts = feeds.map((feed, idx) => {
        const provider = providerMapper(idx);
        return getFeedAccountMeta(provider, feed);
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
    const tokenMap = (await dataStore.account.store.fetch(store)).tokenMap;
    return await exchange.methods.executeDeposit(toBN(options?.executionFee ?? 0)).accounts({
        authority,
        store,
        oracle,
        tokenMap,
        market: createMarketPDA(store, marketToken)[0],
        marketTokenMint: marketToken,
        user,
        deposit,
        receiver: toMarketTokenAccount,
        priceProvider: options.priceProvider ?? PYTH_ID,
    }).remainingAccounts([...feedAccounts, ...swapPathMarkets, ...swapPathMints]).instruction();
};

export const invokeExecuteDeposit = makeInvoke(makeExecuteDepositInstruction, ["authority"]);

const postPriceFeeds = async (
    connection: Connection,
    signer: Keypair,
    feeds: PublicKey[],
    providerMapper: (index: number) => number,
    wait: number = 3000,
) => {
    // Wait for `wait` ms.
    await new Promise(resolve => setTimeout(resolve, wait));
    const hermes = new PriceServiceConnection(
        "https://hermes.pyth.network/",
        { priceFeedRequestConfig: { binary: true } }
    );
    const feedIds = feeds.filter((feed, idx) => getSelectedProvider(providerMapper(idx)) === PriceProvider.Pyth).map(feed => utils.bytes.hex.encode(feed.toBuffer()));
    const data = await hermes.getLatestVaas(feedIds);
    const receiver = new PythSolanaReceiver({
        connection,
        wallet: new Wallet(signer),
    });
    const builder = receiver.newTransactionBuilder({ closeUpdateAccounts: false });
    await builder.addUpdatePriceFeed(data, 0);
    const txs = await receiver.provider.sendAll(await builder.buildVersionedTransactions({ computeUnitPriceMicroLamports: 50000 }), { skipPreflight: true });
    console.log(`updated price feeds with ${txs}`);
};

export const executeDeposit = async (simulate: boolean, ...args: Parameters<typeof invokeExecuteDeposit>) => {
    if (simulate) {
        return ["not executed because this is just a simulation"];
    }
    const [connection, { authority, deposit, options }] = args;
    if (options?.priceProvider?.toBase58() ?? PYTH_ID === PYTH_ID) {
        if (!(authority instanceof Keypair)) {
            throw Error("Currently only support to call with `Keypair`");
        }
        const { feeds, providerMapper } = options?.hints?.deposit ?? await fetchDepositHint(dataStore, deposit);
        await postPriceFeeds(connection, authority as Keypair, feeds, providerMapper);
    }
    return await invokeExecuteDeposit(...args);
};

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
        withdrawal,
        user,
        marketToken: toMarketTokenAccount,
        marketTokenWithdrawalVault: createMarketVaultPDA(store, marketToken)[0],
    }).instruction();
};

export const invokeCancelWithdrawal = makeInvoke(makeCancelWithdrawalInstruction, ["authority"]);

export interface WithdrawalHint {
    user: PublicKey,
    marketTokenMint: PublicKey,
    finalLongTokenReceiver: PublicKey,
    finalShortTokenReceiver: PublicKey,
    finalLongTokenMint: PublicKey,
    finalShortTokenMint: PublicKey,
    feeds: PublicKey[],
    longSwapPath: PublicKey[],
    shortSwapPath: PublicKey[],
    providerMapper: (number) => number | undefined,
}

export type MakeExecuteWithdrawalParams = {
    authority: PublicKey,
    store: PublicKey,
    oracle: PublicKey,
    withdrawal: PublicKey,
    options?: {
        executionFee?: number | bigint,
        priceProvider?: PublicKey,
        hints?: {
            withdrawal?: WithdrawalHint,
        }
    },
};

const fetchWithdrawalHint = async (dataStore: DataStoreProgram, withdrawal: PublicKey) => {
    return await dataStore.account.withdrawal.fetch(withdrawal).then(withdrawal => {
        return {
            user: withdrawal.fixed.user,
            marketTokenMint: withdrawal.fixed.tokens.marketToken,
            finalLongTokenMint: withdrawal.fixed.tokens.finalLongToken,
            finalShortTokenMint: withdrawal.fixed.tokens.finalShortToken,
            finalLongTokenReceiver: withdrawal.fixed.receivers.finalLongTokenReceiver,
            finalShortTokenReceiver: withdrawal.fixed.receivers.finalShortTokenReceiver,
            feeds: withdrawal.dynamic.tokensWithFeed.feeds,
            longSwapPath: withdrawal.dynamic.swap.longTokenSwapPath,
            shortSwapPath: withdrawal.dynamic.swap.shortTokenSwapPath,
            providerMapper: makeProviderMapper(
                [...withdrawal.dynamic.tokensWithFeed.providers],
                withdrawal.dynamic.tokensWithFeed.nums,
            ),
        } satisfies WithdrawalHint as WithdrawalHint;
    });
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
        longSwapPath,
        shortSwapPath,
        providerMapper,
    } = options?.hints?.withdrawal ?? await fetchWithdrawalHint(dataStore, withdrawal);
    const feedAccounts = feeds.map((feed, idx) => {
        return getFeedAccountMeta(providerMapper(idx), feed);
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
    const tokenMap = (await dataStore.account.store.fetch(store)).tokenMap;
    return await exchange.methods.executeWithdrawal(toBN(options?.executionFee ?? 0)).accounts({
        authority,
        store,
        oracle,
        tokenMap,
        withdrawal,
        user,
        market: createMarketPDA(store, marketTokenMint)[0],
        marketTokenMint,
        marketTokenWithdrawalVault: createMarketVaultPDA(store, marketTokenMint)[0],
        finalLongTokenReceiver,
        finalShortTokenReceiver,
        finalLongTokenVault: createMarketVaultPDA(store, finalLongTokenMint)[0],
        finalShortTokenVault: createMarketVaultPDA(store, finalShortTokenMint)[0],
        priceProvider: options.priceProvider ?? PYTH_ID,
    }).remainingAccounts([...feedAccounts, ...swapPathMarkets, ...swapPathMints]).instruction();
};

export const invokeExecuteWithdrawal = makeInvoke(makeExecuteWithdrawalInstruction, ["authority"]);

export const executeWithdrawal = async (simulate: boolean, ...args: Parameters<typeof invokeExecuteWithdrawal>) => {
    if (simulate) {
        return ["not executed because this is just a simulation"];
    }
    const [connection, { authority, withdrawal, options }] = args;
    if (options?.priceProvider?.toBase58() ?? PYTH_ID === PYTH_ID) {
        if (!(authority instanceof Keypair)) {
            throw Error("Currently only support to call with `Keypair`");
        }
        const { feeds, providerMapper } = options?.hints?.withdrawal ?? await fetchWithdrawalHint(dataStore, withdrawal);
        await postPriceFeeds(connection, authority as Keypair, feeds, providerMapper);
    }
    return await invokeExecuteWithdrawal(...args);
};

export interface OrderHint {
    user: PublicKey,
    isDecrease: boolean,
    marketTokenMint: PublicKey,
    position: PublicKey | null,
    feeds: PublicKey[],
    swapPath: PublicKey[],
    finalOutputToken: PublicKey | null,
    secondaryOutputToken: PublicKey | null,
    finalOutputTokenAccount: PublicKey | null,
    secondaryOutputTokenAccount: PublicKey | null,
    isLong: boolean,
    longTokenMint: PublicKey,
    shortTokenMint: PublicKey,
    longTokenAccount: PublicKey,
    shortTokenAccount: PublicKey,
    providerMapper: (number) => number | undefined,
}

export type MakeExecuteOrderParams = {
    authority: PublicKey,
    store: PublicKey,
    oracle: PublicKey,
    order: PublicKey,
    recentTimestamp: bigint | number,
    holding: PublicKey,
    options?: {
        executionFee?: number | bigint,
        priceProvider?: PublicKey,
        hints?: {
            order?: OrderHint,
        }
    },
};

const fetchOrderHint = async (dataStore: DataStoreProgram, orderAddress: PublicKey, store: PublicKey) => {
    const order = await dataStore.account.order.fetch(orderAddress);
    const market = await dataStore.account.market.fetch(order.fixed.market);
    return {
        user: order.fixed.user,
        isDecrease: Boolean(order.fixed.params.kind.marketDecrease),
        marketTokenMint: order.fixed.tokens.marketToken,
        position: order.fixed.position ?? null,
        finalOutputToken: order.fixed.tokens.finalOutputToken ?? null,
        secondaryOutputToken: order.fixed.tokens.secondaryOutputToken ?? null,
        finalOutputTokenAccount: order.fixed.receivers.finalOutputTokenAccount ?? null,
        secondaryOutputTokenAccount: order.fixed.receivers.secondaryOutputTokenAccount ?? null,
        longTokenAccount: order.fixed.receivers.longTokenAccount,
        shortTokenAccount: order.fixed.receivers.shortTokenAccount,
        isLong: order.fixed.params.isLong,
        longTokenMint: market.meta.longTokenMint,
        shortTokenMint: market.meta.shortTokenMint,
        feeds: order.prices.feeds,
        swapPath: order.swap.longTokenSwapPath,
        providerMapper: makeProviderMapper([...order.prices.providers], order.prices.nums),
    } satisfies OrderHint as OrderHint;
}

export const makeExecuteOrderInstruction = async ({
    authority,
    store,
    oracle,
    order,
    recentTimestamp,
    holding,
    options,
}: MakeExecuteOrderParams) => {
    const {
        user,
        isDecrease,
        isLong,
        marketTokenMint,
        position,
        finalOutputToken,
        finalOutputTokenAccount,
        secondaryOutputToken,
        secondaryOutputTokenAccount,
        longTokenAccount,
        shortTokenAccount,
        longTokenMint,
        shortTokenMint,
        feeds,
        swapPath,
        providerMapper,
    } = options?.hints?.order ?? await fetchOrderHint(dataStore, order, store);
    const [onlyOrderKeeper] = createRolesPDA(store, authority);
    const feedAccounts = feeds.map((pubkey, idx) => {
        return getFeedAccountMeta(providerMapper(idx), pubkey);
    });
    const swapMarkets = swapPath.map(mint => {
        return {
            pubkey: createMarketPDA(store, mint)[0],
            isSigner: false,
            isWritable: true,
        }
    });
    const swapMarketMints = swapPath.map(pubkey => {
        return {
            pubkey,
            isSigner: false,
            isWritable: false,
        }
    });
    const pnlTokenMint = isLong ? longTokenMint : shortTokenMint;
    const tokenMap = (await dataStore.account.store.fetch(store)).tokenMap;
    return await exchange.methods.executeOrder(toBN(recentTimestamp), toBN(options?.executionFee ?? 0)).accounts({
        authority,
        store,
        oracle,
        tokenMap,
        market: createMarketPDA(store, marketTokenMint)[0],
        marketTokenMint,
        order,
        position,
        user,
        finalOutputTokenAccount,
        secondaryOutputTokenAccount,
        finalOutputTokenVault: finalOutputTokenAccount ? createMarketVaultPDA(store, finalOutputToken)[0] : null,
        secondaryOutputTokenVault: secondaryOutputTokenAccount ? createMarketVaultPDA(store, secondaryOutputToken)[0] : null,
        longTokenAccount,
        shortTokenAccount,
        longTokenVault: findMarketVaultPDA(store, longTokenMint)[0],
        shortTokenVault: findMarketVaultPDA(store, shortTokenMint)[0],
        claimableLongTokenAccountForUser: isDecrease ? findClaimableAccountPDA(store, longTokenMint, user, getTimeKey(recentTimestamp, TIME_WINDOW))[0] : null,
        claimableShortTokenAccountForUser: isDecrease ? findClaimableAccountPDA(store, shortTokenMint, user, getTimeKey(recentTimestamp, TIME_WINDOW))[0] : null,
        claimablePnlTokenAccountForHolding: isDecrease ? findClaimableAccountPDA(store, pnlTokenMint, holding, getTimeKey(recentTimestamp, TIME_WINDOW))[0] : null,
        priceProvider: options.priceProvider ?? PYTH_ID,
    }).remainingAccounts([...feedAccounts, ...swapMarkets, ...swapMarketMints]).instruction();
};

export const invokeExecuteOrder = makeInvoke(makeExecuteOrderInstruction, ["authority"]);

export const executeOrder = async (simulate: boolean, ...args: Parameters<typeof invokeExecuteOrder>) => {
    if (simulate) {
        return ["not executed because this is just a simulation"];
    }
    const [connection, { authority, order, store, recentTimestamp, holding, options }] = args;
    const { user, longTokenMint, shortTokenMint, isLong, isDecrease, feeds, providerMapper } = options?.hints?.order ?? await fetchOrderHint(dataStore, order, store);
    if (options?.priceProvider?.toBase58() ?? PYTH_ID === PYTH_ID) {
        if (!(authority instanceof Keypair)) {
            throw Error("Currently only support to call with `Keypair`");
        }
        await postPriceFeeds(connection, authority as Keypair, feeds, providerMapper);
    }
    if (isDecrease) {
        await invokeUseClaimableAccount(dataStore, {
            authority,
            store,
            user,
            mint: longTokenMint,
            timestamp: recentTimestamp,
        });
        await invokeUseClaimableAccount(dataStore, {
            authority,
            store,
            user,
            mint: shortTokenMint,
            timestamp: recentTimestamp,
        });
        await invokeUseClaimableAccount(dataStore, {
            authority,
            store,
            user: holding,
            mint: isLong ? longTokenMint : shortTokenMint,
            timestamp: recentTimestamp,
        });
        const signature = await invokeExecuteOrder(...args);
        await invokeCloseEmptyClaimableAccount(dataStore, {
            authority,
            store,
            user,
            mint: longTokenMint,
            timestamp: recentTimestamp,
        });
        await invokeCloseEmptyClaimableAccount(dataStore, {
            authority,
            store,
            user,
            mint: shortTokenMint,
            timestamp: recentTimestamp,
        });
        await invokeCloseEmptyClaimableAccount(dataStore, {
            authority,
            store,
            user: holding,
            mint: isLong ? longTokenMint : shortTokenMint,
            timestamp: recentTimestamp,
        });
        return signature;
    } else {
        return await invokeExecuteOrder(...args);
    }
};

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
