import { workspace, Program, BN, utils } from "@coral-xyz/anchor";
import { Exchange } from "../target/types/exchange";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { createDepositPDA, createMarketPDA, createMarketTokenMintPDA, createMarketVaultPDA, createOrderPDA, createPositionPDA, createRolesPDA, createTokenConfigMapPDA, createWithdrawalPDA, dataStore } from "./data";
import { TOKEN_PROGRAM_ID, getAccount } from "@solana/spl-token";
import { BTC_TOKEN_MINT, SOL_TOKEN_MINT } from "./token";
import { toBN } from "./number";
import { CHAINLINK_ID } from "./external";
import { IxWithOutput, makeInvoke } from "./invoke";

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
    const [authority] = createControllerPDA(store);
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

export const invokeCreateDeposit = makeInvoke(makeCreateDepositInstruction, ["payer"]);

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
    const {
        user,
        marketToken,
        toMarketTokenAccount,
        feeds,
        longSwapPath,
        shortSwapPath,
    } = options?.hints?.deposit ?? await exchange.account.deposit.fetch(deposit).then(deposit => {
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
    store,
    payer,
    marketToken,
    amount,
    fromMarketTokenAccount,
    toLongTokenAccount,
    toShortTokenAccount,
    options,
}: MakeCreateWithdrawalParams) => {
    const [authority] = createControllerPDA(store);
    const withdrawalNonce = options?.nonce ?? Keypair.generate().publicKey.toBuffer();
    const [withdrawalAddress] = createWithdrawalPDA(store, payer, withdrawalNonce);
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
    }).remainingAccounts([...longSwapPath, ...shortSwapPath].map(token => {
        return {
            pubkey: createMarketPDA(store, token)[0],
            isSigner: false,
            isWritable: false,
        };
    })).instruction();

    return [instruction, withdrawalAddress] as IxWithOutput<PublicKey>;
};

export const invokeCreateWithdrawal = makeInvoke(makeCreateWithdrawalInstruction, ["payer"]);

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
                longSwapPath: PublicKey[],
                shortSwapPath: PublicKey[],
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
        longSwapPath,
        shortSwapPath,
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
                longSwapPath: withdrawal.dynamic.swap.longTokenSwapPath,
                shortSwapPath: withdrawal.dynamic.swap.shortTokenSwapPath,
            }
        }));
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
    return await exchange.methods.executeWithdrawal(toBN(options?.executionFee ?? 0)).accounts({
        authority,
        store,
        onlyOrderKeeper: createRolesPDA(store, authority)[0],
        dataStoreProgram: dataStore.programId,
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
    }).remainingAccounts([...feedAccounts, ...swapPathMarkets, ...swapPathMints]).instruction();
};

export const invokeExecuteWithdrawal = makeInvoke(makeExecuteWithdrawalInstruction, ["authority"]);

export type MakeCreateOrderParams = {
    store: PublicKey,
    payer: PublicKey,
    orderType: "marketSwap" | "marketIncrease" | "marketDecrease" | "liquidation",
    marketToken: PublicKey,
    isCollateralTokenLong: boolean,
    initialCollateralDeltaAmount: number | bigint,
    isLong?: boolean,
    sizeDeltaUsd?: number | bigint,
    fromTokenAccount?: PublicKey,
    toTokenAccount?: PublicKey,
    secondaryTokenAccount?: PublicKey,
    options?: {
        nonce?: Buffer,
        executionFee?: number | bigint,
        swapPath?: PublicKey[],
        minOutputAmount?: number | bigint,
        acceptablePrice?: number | bigint,
        hints?: {
            initialToken?: PublicKey,
            collateralToken?: PublicKey,
        }
    },
};

export const makeCreateOrderInstruction = async ({
    store,
    payer,
    orderType,
    marketToken,
    isCollateralTokenLong,
    initialCollateralDeltaAmount,
    isLong,
    sizeDeltaUsd,
    fromTokenAccount,
    toTokenAccount,
    secondaryTokenAccount,
    options,
}: MakeCreateOrderParams) => {
    const [market] = createMarketPDA(store, marketToken);

    const getCollateralToken = async (isCollateralTokenLong: boolean) => {
        const meta = (await dataStore.account.market.fetch(market)).meta;
        return isCollateralTokenLong ? meta.longTokenMint : meta.shortTokenMint;
    };
    const collateralToken = options?.hints?.collateralToken ?? await getCollateralToken(isCollateralTokenLong);

    let kind;
    let position: PublicKey | null = null;

    switch (orderType) {
        case "marketSwap":
            kind = {
                marketSwap: {},
            };
            break;
        case "marketIncrease":
            kind = {
                marketIncrease: {},
            };
            if (isLong === undefined) {
                throw "position side must be provided";
            }
            position = createPositionPDA(store, payer, marketToken, collateralToken, isLong)[0]
            break;
        case "marketDecrease":
            kind = {
                marketDecrease: {},
            };
            if (isLong === undefined) {
                throw "position side must be provided";
            }
            position = createPositionPDA(store, payer, marketToken, collateralToken, isLong)[0]
            break;
        case "liquidation":
            kind = {
                liquidation: {},
            };
            if (isLong === undefined) {
                throw "position side must be provided";
            }
            position = createPositionPDA(store, payer, marketToken, collateralToken, isLong)[0]
            break;
    }

    const nonce = options?.nonce ?? Keypair.generate().publicKey.toBuffer();
    const swapPath = options?.swapPath ?? [];
    const [authority] = createControllerPDA(store);
    const [onlyController] = createRolesPDA(store, authority);
    const [order] = createOrderPDA(store, payer, nonce);
    const acceptablePrice = options?.acceptablePrice;
    const instruction = await exchange.methods.createOrder(
        [...nonce],
        {
            order: {
                kind,
                minOutputAmount: toBN(options?.minOutputAmount ?? 0),
                sizeDeltaUsd: toBN(sizeDeltaUsd ?? 0),
                initialCollateralDeltaAmount: toBN(initialCollateralDeltaAmount),
                acceptablePrice: acceptablePrice ? toBN(acceptablePrice) : null,
                isLong,
            },
            outputToken: collateralToken,
            uiFeeReceiver: Keypair.generate().publicKey,
            executionFee: toBN(options?.executionFee ?? 0),
            swapLength: swapPath.length,
        },
    ).accounts({
        authority,
        store,
        onlyController,
        payer,
        order,
        position,
        tokenConfigMap: createTokenConfigMapPDA(store)[0],
        market,
        initialCollateralTokenAccount: fromTokenAccount ?? null,
        finalOutputTokenAccount: toTokenAccount ?? null,
        secondaryOutputTokenAccount: secondaryTokenAccount ?? null,
        initialCollateralTokenVault: (await getDepositVault(exchange.provider.connection, store, fromTokenAccount, options?.hints?.initialToken)) ?? null,
        dataStoreProgram: dataStore.programId,
    }).remainingAccounts(swapPath.map(mint => {
        return {
            pubkey: createMarketPDA(store, mint)[0],
            isSigner: false,
            isWritable: false,
        };
    })).instruction();

    return [instruction, order] as IxWithOutput<PublicKey>;
};

export const invokeCreateOrder = makeInvoke(makeCreateOrderInstruction, ["payer"]);

export type MakeExecuteOrderParams = {
    authority: PublicKey,
    store: PublicKey,
    oracle: PublicKey,
    order: PublicKey,
    options?: {
        executionFee?: number | bigint,
        hints?: {
            order?: {
                user: PublicKey,
                marketTokenMint: PublicKey,
                position: PublicKey | null,
                feeds: PublicKey[],
                swapPath: PublicKey[],
                finalOutputToken: PublicKey | null,
                secondaryOutputToken: PublicKey | null,
                finalOutputTokenAccount: PublicKey | null,
                secondaryOutputTokenAccount: PublicKey | null,
            }
        }
    },
};

export const makeExecuteOrderInstruction = async ({
    authority,
    store,
    oracle,
    order,
    options,
}: MakeExecuteOrderParams) => {
    const {
        user,
        marketTokenMint,
        position,
        finalOutputToken,
        finalOutputTokenAccount,
        secondaryOutputToken,
        secondaryOutputTokenAccount,
        feeds,
        swapPath,
    } = options?.hints?.order ?? await dataStore.account.order.fetch(order).then(order => {
        return {
            user: order.fixed.user,
            marketTokenMint: order.fixed.tokens.marketToken,
            position: order.fixed.position ?? null,
            finalOutputToken: order.fixed.tokens.finalOutputToken ?? null,
            secondaryOutputToken: order.fixed.tokens.secondaryOutputToken ?? null,
            finalOutputTokenAccount: order.fixed.receivers.finalOutputTokenAccount ?? null,
            secondaryOutputTokenAccount: order.fixed.receivers.secondaryOutputTokenAccount ?? null,
            feeds: order.prices.feeds,
            swapPath: order.swap.longTokenSwapPath,
        };
    });
    const [onlyOrderKeeper] = createRolesPDA(store, authority);
    const feedAccounts = feeds.map(pubkey => {
        return {
            pubkey,
            isSigner: false,
            isWritable: false,
        }
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
    return await exchange.methods.executeOrder(toBN(options?.executionFee ?? 0)).accounts({
        authority,
        onlyOrderKeeper,
        store,
        oracle,
        tokenConfigMap: createTokenConfigMapPDA(store)[0],
        market: createMarketPDA(store, marketTokenMint)[0],
        marketTokenMint,
        order,
        position,
        user,
        finalOutputTokenAccount,
        secondaryOutputTokenAccount,
        finalOutputTokenVault: finalOutputTokenAccount ? createMarketVaultPDA(store, finalOutputToken)[0] : null,
        secondaryOutputTokenVault: secondaryOutputTokenAccount ? createMarketVaultPDA(store, secondaryOutputToken)[0] : null,
        dataStoreProgram: dataStore.programId,
        chainlinkProgram: CHAINLINK_ID,
    }).remainingAccounts([...feedAccounts, ...swapMarkets, ...swapMarketMints]).instruction();
};

export const invokeExecuteOrder = makeInvoke(makeExecuteOrderInstruction, ["authority"]);

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
