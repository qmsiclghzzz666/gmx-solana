import * as anchor from "@coral-xyz/anchor";
import { Oracle } from "../target/types/oracle";
import { IDL as chainlinkIDL } from "../external-programs/chainlink-store";
import { getProvider } from "../utils/fixtures";

describe("oracle", () => {
    const provider = getProvider();

    const chainlinkID = "HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny";
    const oracle = anchor.workspace.Oracle as anchor.Program<Oracle>;
    const chainlink = new anchor.Program(chainlinkIDL, chainlinkID);

    const mockFeedAccount = anchor.web3.Keypair.generate();

    it("should work", async () => {
        try {
            const round = await oracle.methods.initialize().accounts({
                feed: "Cv4T27XbjVoKUYwP72NQQanvZeA7W4YF9L4EnYT9kx5o",
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
});
