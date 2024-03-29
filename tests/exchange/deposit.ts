import { BN } from "@coral-xyz/anchor";
import { ComputeBudgetProgram, Keypair, PublicKey, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { createDepositPDA, createMarketTokenMintPDA, createNoncePDA, createRolesPDA, createTokenConfigPDA } from "../../utils/data";
import { getAddresses, getExternalPrograms, getMarkets, getPrograms, getProvider, getUsers, expect } from "../../utils/fixtures";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { BTC_FEED, USDC_FEED } from "../../utils/token";

describe("exchange: deposit", () => {
    const provider = getProvider();
    const { exchange, dataStore, oracle } = getPrograms();
    const { chainlink } = getExternalPrograms();
    const { signer0, user0 } = getUsers();

    let dataStoreAddress: PublicKey;
    let user0FakeTokenAccount: PublicKey;
    let user0UsdGTokenAccount: PublicKey;
    let user0FakeFakeUsdGTokenAccount: PublicKey;
    let fakeTokenVault: PublicKey;
    let usdGVault: PublicKey;
    let marketFakeFakeUsdG: PublicKey;
    let roles: PublicKey;
    let nonce: PublicKey;
    let oracleAddress: PublicKey;
    let fakeTokenMint: PublicKey;
    let usdGTokenMint: PublicKey;

    before(async () => {
        ({
            dataStoreAddress,
            user0FakeTokenAccount,
            user0UsdGTokenAccount,
            user0FakeFakeUsdGTokenAccount,
            fakeTokenVault,
            usdGVault,
            oracleAddress,
            fakeTokenMint,
            usdGTokenMint,
        } = await getAddresses());
        ({ marketFakeFakeUsdG } = await getMarkets());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
        [nonce] = createNoncePDA(dataStoreAddress);
    });

    it("create and execute deposit", async () => {
        const depositNonce = await dataStore.methods.getNonceBytes().accounts({ nonce }).view();
        const [deposit] = createDepositPDA(dataStoreAddress, user0.publicKey, depositNonce);
        {
            const ix = await exchange.methods.createDeposit(
                [...depositNonce],
                {
                    uiFeeReceiver: Keypair.generate().publicKey,
                    executionFee: new BN(0),
                    longTokenSwapPath: [],
                    shortTokenSwapPath: [],
                    initialLongTokenAmount: new BN(1_000_000_000),
                    initialShortTokenAmount: new BN(100_000_000),
                    minMarketToken: new BN(0),
                    shouldUnwrapNativeToken: false,
                },
            ).accounts({
                market: marketFakeFakeUsdG,
                authority: signer0.publicKey,
                store: dataStoreAddress,
                onlyController: roles,
                dataStoreProgram: dataStore.programId,
                deposit,
                payer: user0.publicKey,
                receiver: user0FakeFakeUsdGTokenAccount,
                initialLongToken: user0FakeTokenAccount,
                initialShortToken: user0UsdGTokenAccount,
                longTokenDepositVault: fakeTokenVault,
                shortTokenDepositVault: usdGVault,
                tokenProgram: TOKEN_PROGRAM_ID,
            }).postInstructions([
                await dataStore.methods.incrementNonce().accounts({
                    authority: signer0.publicKey,
                    store: dataStoreAddress,
                    onlyController: roles,
                    nonce,
                }).instruction(),
            ]).signers([signer0, user0]).instruction();
            const txId = await sendAndConfirmTransaction(provider.connection, new Transaction().add(ix), [user0, signer0]);
            console.log("created at", txId);
        }
        try {
            const ix = await exchange.methods.executeDeposit(new BN(5001)).accounts({
                authority: signer0.publicKey,
                store: dataStoreAddress,
                dataStoreProgram: dataStore.programId,
                onlyOrderKeeper: roles,
                oracleProgram: oracle.programId,
                oracle: oracleAddress,
                chainlinkProgram: chainlink.programId,
                deposit,
                user: user0.publicKey,
                receiver: user0FakeFakeUsdGTokenAccount,
                market: marketFakeFakeUsdG,
                marketTokenMint: createMarketTokenMintPDA(dataStoreAddress, fakeTokenMint, fakeTokenMint, usdGTokenMint)[0],
            }).remainingAccounts([
                {
                    pubkey: createTokenConfigPDA(dataStoreAddress, fakeTokenMint.toBase58())[0],
                    isSigner: false,
                    isWritable: false,
                },
                {
                    pubkey: BTC_FEED,
                    isSigner: false,
                    isWritable: false,
                },
                {
                    pubkey: createTokenConfigPDA(dataStoreAddress, usdGTokenMint.toBase58())[0],
                    isSigner: false,
                    isWritable: false,
                },
                {
                    pubkey: USDC_FEED,
                    isSigner: false,
                    isWritable: false,
                },
            ]).instruction();
            const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
                units: 800_000
            });
            const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({
                microLamports: 1,
            });
            const tx = new Transaction()
                .add(modifyComputeUnits)
                .add(addPriorityFee)
                .add(ix);
            const txId = await sendAndConfirmTransaction(provider.connection, tx, [signer0]);
            console.log(`executed at`, txId);
        } catch (error) {
            console.log(error);
            throw error;
        } finally {
            const afterExecution = await dataStore.account.oracle.fetch(oracleAddress);
            expect(afterExecution.primary.prices.length).equals(0);
            const market = await dataStore.account.market.fetch(marketFakeFakeUsdG);
            console.log("pools", market.pools);
        }
    });

    it("create and cancel deposit", async () => {
        const depositNonce = Keypair.generate().publicKey.toBuffer();
        const [deposit] = createDepositPDA(dataStoreAddress, user0.publicKey, depositNonce);
        {
            const ix = await exchange.methods.createDeposit(
                [...depositNonce],
                {
                    uiFeeReceiver: Keypair.generate().publicKey,
                    executionFee: new BN(0),
                    longTokenSwapPath: [],
                    shortTokenSwapPath: [],
                    initialLongTokenAmount: new BN(2_000_000_000),
                    initialShortTokenAmount: new BN(200_000_000),
                    minMarketToken: new BN(0),
                    shouldUnwrapNativeToken: false,
                },
            ).accounts({
                market: marketFakeFakeUsdG,
                authority: signer0.publicKey,
                store: dataStoreAddress,
                onlyController: roles,
                dataStoreProgram: dataStore.programId,
                deposit,
                payer: user0.publicKey,
                receiver: user0FakeFakeUsdGTokenAccount,
                initialLongToken: user0FakeTokenAccount,
                initialShortToken: user0UsdGTokenAccount,
                longTokenDepositVault: fakeTokenVault,
                shortTokenDepositVault: usdGVault,
                tokenProgram: TOKEN_PROGRAM_ID,
            }).signers([signer0, user0]).instruction();
            const txId = await sendAndConfirmTransaction(provider.connection, new Transaction().add(ix), [user0, signer0]);
            console.log("created at", txId);
        }
        try {
            const ix = await exchange.methods.cancelDeposit(new BN(5000)).accounts({
                authority: signer0.publicKey,
                store: dataStoreAddress,
                dataStoreProgram: dataStore.programId,
                onlyController: roles,
                deposit,
                user: user0.publicKey,
                initialLongToken: user0FakeTokenAccount,
                initialShortToken: user0UsdGTokenAccount,
                longTokenDepositVault: fakeTokenVault,
                shortTokenDepositVault: usdGVault,
                tokenProgram: TOKEN_PROGRAM_ID,
            }).instruction();
            const tx = new Transaction().add(ix);
            const txId = await sendAndConfirmTransaction(provider.connection, tx, [signer0]);
            console.log(`cancelled at`, txId);
        } catch (error) {
            console.log(error);
            throw error;
        } finally {
            const afterExecution = await dataStore.account.oracle.fetch(oracleAddress);
            expect(afterExecution.primary.prices.length).equals(0);
            const market = await dataStore.account.market.fetch(marketFakeFakeUsdG);
            console.log("pools", market.pools);
        }
    });
});
