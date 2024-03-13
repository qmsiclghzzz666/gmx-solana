import * as anchor from "@coral-xyz/anchor";
import { Oracle } from "../target/types/oracle";
import { IDL as chainlinkIDL } from "../external-programs/chainlink-store";
import { getAddresses, getProvider, getUsers } from "../utils/fixtures";
import { BTC_FEED, BTC_TOKEN_MINT, SOL_FEED, SOL_TOKEN_MINT, createAddressPDA, createPriceFeedKey, createTokenConfigPDA } from "../utils/data";
import { createControllerPDA } from "../utils/role";
import { expect } from "chai";

describe("oracle", () => {
    const provider = getProvider();

    const chainlinkID = "HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny";
    const oracle = anchor.workspace.Oracle as anchor.Program<Oracle>;
    const chainlink = new anchor.Program(chainlinkIDL, chainlinkID);

    const { dataStoreAddress, roleStoreAddress, oracleAddress } = getAddresses();
    const { signer0 } = getUsers();

    const mockFeedAccount = anchor.web3.Keypair.generate();

    it("create a new price feed", async () => {
        try {
            const createFeedTx = await chainlink.methods.createFeed("FOO", 1, 2, 3).accounts({
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
        const [onlyController] = createControllerPDA(roleStoreAddress, signer0.publicKey);
        await oracle.methods.setPricesFromPriceFeed([
            BTC_TOKEN_MINT,
            SOL_TOKEN_MINT,
        ]).accounts({
            store: dataStoreAddress,
            authority: signer0.publicKey,
            chainlinkProgram: chainlinkID,
            onlyController,
            oracle: oracleAddress,
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
            authority: signer0.publicKey,
            onlyController,
            oracle: oracleAddress,
        }).signers([signer0]).rpc();
        const clearedData = await oracle.account.oracle.fetch(oracleAddress);
        expect(clearedData.primary.prices.length).to.equal(0);
        expect(clearedData.primary.tokens.length).to.equal(0);
    });
});
