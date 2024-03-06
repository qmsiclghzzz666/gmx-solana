import * as anchor from "@coral-xyz/anchor";
import { Oracle } from "../target/types/oracle";
import { IDL as chainlinkIDL } from "../external-programs/chainlink-store";
import { getAddresses, getProvider, getUsers } from "../utils/fixtures";
import { BTC_FEED, BTC_TOKEN, SOL_FEED, SOL_TOKEN, createAddressPDA, createPriceFeedKey } from "../utils/data";
import { createControllerPDA } from "../utils/role";

describe("oracle", () => {
    const provider = getProvider();

    const chainlinkID = "HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny";
    const oracle = anchor.workspace.Oracle as anchor.Program<Oracle>;
    const chainlink = new anchor.Program(chainlinkIDL, chainlinkID);

    const { dataStoreAddress, roleStoreAddress, oracleAddress } = getAddresses();
    const { signer0 } = getUsers();

    const mockFeedAccount = anchor.web3.Keypair.generate();

    it("get price from the given feed", async () => {
        try {
            const round = await oracle.methods.getPriceFromFeed().accounts({
                feed: BTC_FEED,
                chainlinkProgram: chainlinkID,
            }).view();
            console.log(`got round of slot ${round.slot}, answer: ${round.answer}, feed ts: ${round.timestamp}, sys ts: ${round.sysTimestamp}`, round);
        } catch (error) {
            console.log(error);
            throw error;
        }

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
            console.log("create feed:", createFeedTx);
        } catch (error) {
            console.error(error);
            throw error;
        }
    });

    it("set price from feed", async () => {
        await oracle.methods.setPricesFromPriceFeed([
            BTC_TOKEN,
            SOL_TOKEN,
        ]).accounts({
            store: dataStoreAddress,
            authority: signer0.publicKey,
            chainlinkProgram: chainlinkID,
            role: createControllerPDA(roleStoreAddress, signer0.publicKey)[0],
            oracle: oracleAddress,
        }).remainingAccounts([
            {
                pubkey: createAddressPDA(dataStoreAddress, createPriceFeedKey(BTC_TOKEN))[0],
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: BTC_FEED,
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: createAddressPDA(dataStoreAddress, createPriceFeedKey(SOL_TOKEN))[0],
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: SOL_FEED,
                isSigner: false,
                isWritable: false,
            },
        ]).signers([signer0]).rpc();
        const oracleData = await oracle.account.oracle.fetch(oracleAddress);
        console.log(oracleData.primary.prices);
        console.log(oracleData.primary.tokens);
    });
});
