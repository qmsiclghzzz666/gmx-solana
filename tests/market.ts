import * as anchor from "@coral-xyz/anchor";
import { expect, getAddresses, getPrograms, getTokenMints, getUsers } from "../utils/fixtures";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createMarketKeeperPDA } from "../utils/role";
import { createMarketPDA } from "../utils/data";
import { createLongTokenPDA, createMarketTokenMintPDA, createShortTokenPDA, getMarketTokenAuthority } from "../utils/market";

describe("market", () => {
    const { market } = getPrograms();
    const { signer0 } = getUsers();
    const { roleStoreAddress, dataStoreAddress } = getAddresses();
    const { dataStore } = getPrograms();
    const { BTC_TOKEN_MINT, SOL_TOKEN_MINT } = getTokenMints();

    const indexTokenMint = Keypair.generate().publicKey;
    const longTokenMint = BTC_TOKEN_MINT;
    const shortTokenMint = SOL_TOKEN_MINT;

    it("create market", async () => {
        const [marketTokenMint] = createMarketTokenMintPDA(dataStoreAddress, indexTokenMint, longTokenMint, shortTokenMint);
        const [longToken] = createLongTokenPDA(marketTokenMint);
        const [shortToken] = createShortTokenPDA(marketTokenMint);
        const [marketAuthority] = getMarketTokenAuthority();
        await market.methods.createMarket(indexTokenMint).accounts({
            authority: signer0.publicKey,
            onlyMarketKeeper: createMarketKeeperPDA(roleStoreAddress, signer0.publicKey)[0],
            dataStore: dataStoreAddress,
            market: createMarketPDA(dataStoreAddress, marketTokenMint)[0],
            marketTokenMint,
            longTokenMint,
            shortTokenMint,
            longToken,
            shortToken,
            marketAuthority,
            dataStoreProgram: dataStore.programId,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        }).signers([signer0]).rpc();
    });
});
