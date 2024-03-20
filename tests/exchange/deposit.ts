import { BN } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createDepositPDA, createNoncePDA, createRolesPDA } from "../../utils/data";
import { getAddresses, getMarkets, getPrograms, getUsers } from "../../utils/fixtures";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

describe("exchange: deposit", () => {
    const { exchange, dataStore } = getPrograms();
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

    before(async () => {
        ({
            dataStoreAddress,
            user0FakeTokenAccount,
            user0UsdGTokenAccount,
            user0FakeFakeUsdGTokenAccount,
            fakeTokenVault,
            usdGVault,
        } = await getAddresses());
        ({ marketFakeFakeUsdG } = await getMarkets());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
        [nonce] = createNoncePDA(dataStoreAddress);
    });

    it("create deposit", async () => {
        const depositNonce = await dataStore.methods.getNonceBytes().accounts({ nonce }).view();
        const [deposit] = createDepositPDA(dataStoreAddress, user0.publicKey, depositNonce);
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
        console.log(tx);
    });
});
