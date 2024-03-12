import * as anchor from "@coral-xyz/anchor";
import { expect, getAddresses, getPrograms, getUsers } from "../utils/fixtures";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createMarketKeeperPDA } from "../utils/role";
import { createMarketPDA } from "../utils/data";
import { createMarketTokenPDA, getMarketTokenAuthority } from "../utils/market";

describe("market", () => {
    const { market } = getPrograms();
    const { signer0 } = getUsers();
    const { roleStoreAddress, dataStoreAddress } = getAddresses();
    const { dataStore } = getPrograms();

    const indexToken = Keypair.generate().publicKey;
    const longToken = Keypair.generate().publicKey;
    const shortToken = Keypair.generate().publicKey;

    it("create market", async () => {
        const [marketToken] = createMarketTokenPDA(dataStoreAddress, indexToken, longToken, shortToken);
        const [marketTokenAuthority] = getMarketTokenAuthority();
        await market.methods.createMarket(
            indexToken,
            longToken,
            shortToken,
        ).accounts({
            authority: signer0.publicKey,
            onlyMarketKeeper: createMarketKeeperPDA(roleStoreAddress, signer0.publicKey)[0],
            dataStore: dataStoreAddress,
            market: createMarketPDA(dataStoreAddress, marketToken)[0],
            marketToken,
            marketTokenAuthority,
            dataStoreProgram: dataStore.programId,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        }).signers([signer0]).rpc();
    });
});
