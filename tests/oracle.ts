import * as anchor from "@coral-xyz/anchor";
import { getAddresses, getExternalPrograms, getPrograms, getProvider, getUsers, expect } from "../utils/fixtures";
import { BTC_FEED, BTC_TOKEN_MINT, SOL_FEED, SOL_TOKEN_MINT, createRolesPDA, createTokenConfigPDA, dataStore } from "../utils/data";

describe("oracle", () => {
    const provider = getProvider();

    const { chainlink } = getExternalPrograms();
    const { oracle } = getPrograms();

    const { dataStoreAddress, oracleAddress } = getAddresses();
    const { signer0 } = getUsers();

    const [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);

    const mockFeedAccount = anchor.web3.Keypair.generate();

    it("create a new price feed", async () => {
        try {
            await chainlink.methods.createFeed("FOO", 1, 2, 3).accounts({
                feed: mockFeedAccount.publicKey,
                authority: provider.wallet.publicKey,
            }).signers([mockFeedAccount]).preInstructions([
                // @ts-ignore: ignore because the field name of `transmissions` account generated is wrong.
                await chainlink.account.transmissions.createInstruction(
                    mockFeedAccount,
                    8 + 192 + (3 + 3) * 48
                ),
            ]).rpc();
        } catch (error) {
            console.error(error);
            throw error;
        }
    });

    it("set price from feed and then clear", async () => {
        await oracle.methods.setPricesFromPriceFeed([
            BTC_TOKEN_MINT,
            SOL_TOKEN_MINT,
        ]).accounts({
            store: dataStoreAddress,
            authority: signer0.publicKey,
            chainlinkProgram: chainlink.programId,
            onlyController: roles,
            oracle: oracleAddress,
            dataStoreProgram: dataStore.programId,
        }).remainingAccounts([
            {
                pubkey: createTokenConfigPDA(dataStoreAddress, BTC_TOKEN_MINT.toBase58())[0],
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: BTC_FEED,
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: createTokenConfigPDA(dataStoreAddress, SOL_TOKEN_MINT.toBase58())[0],
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: SOL_FEED,
                isSigner: false,
                isWritable: false,
            },
        ]).signers([signer0]).rpc();
        const setData = await oracle.account.oracle.fetch(oracleAddress);
        expect(setData.primary.prices.length).to.equal(2);
        expect(setData.primary.tokens).to.eql([
            SOL_TOKEN_MINT,
            BTC_TOKEN_MINT,
        ]);

        await oracle.methods.clearAllPrices().accounts({
            store: dataStoreAddress,
            authority: signer0.publicKey,
            onlyController: roles,
            oracle: oracleAddress,
            dataStoreProgram: dataStore.programId,
        }).signers([signer0]).rpc();
        const clearedData = await oracle.account.oracle.fetch(oracleAddress);
        expect(clearedData.primary.prices.length).to.equal(0);
        expect(clearedData.primary.tokens.length).to.equal(0);
    });
});
