import * as anchor from "@coral-xyz/anchor";
import { Oracle } from "../target/types/oracle";

describe("oracle", () => {
    const oracle = anchor.workspace.Oracle as anchor.Program<Oracle>;

    it("should work", async () => {
        const answer = await oracle.methods.initialize().accounts({
            feed: "Cv4T27XbjVoKUYwP72NQQanvZeA7W4YF9L4EnYT9kx5o",
            chainlink: "HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny",
        }).view() as anchor.BN;
        console.log(`got answer: ${answer}`);
    });
});
