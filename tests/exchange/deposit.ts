import { BN } from "@coral-xyz/anchor";
import { ComputeBudgetProgram, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import { createDepositPDA, createNoncePDA, createRolesPDA, createTokenConfigPDA } from "../../utils/data";
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

    it("create deposit", async () => {
        const depositNonce = await dataStore.methods.getNonceBytes().accounts({ nonce }).view();
        const [deposit] = createDepositPDA(dataStoreAddress, user0.publicKey, depositNonce);
        {
            const tx = await exchange.methods.createDeposit(
                [...depositNonce],
                {
                    uiFeeReceiver: Keypair.generate().publicKey,
                    longTokenSwapPath: [],
                    shortTokenSwapPath: [],
                    initialLongTokenAmount: new BN(1),
                    initialShortTokenAmount: new BN(1),
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
            ]).signers([signer0, user0]).rpc();
            console.log("created at", tx);
        }
        try {
            const ix = await exchange.methods.executeDeposit().accounts({
                authority: signer0.publicKey,
                store: dataStoreAddress,
                dataStoreProgram: dataStore.programId,
                deposit,
                onlyOrderKeeper: roles,
                oracleProgram: oracle.programId,
                oracle: oracleAddress,
                chainlinkProgram: chainlink.programId,
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
                units: 200_000
            });
            const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({
                microLamports: 1,
            });
            const tx = await provider.sendAndConfirm(
                new Transaction()
                    .add(modifyComputeUnits)
                    .add(addPriorityFee)
                    .add(ix),
                [signer0],
            );
            console.log("executed at", tx);
        } catch (error) {
            throw error;
        } finally {
            const afterExecution = await dataStore.account.oracle.fetch(oracleAddress);
            expect(afterExecution.primary.prices.length).equals(0);
        }
    });
});
