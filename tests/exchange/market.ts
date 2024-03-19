import { getAddresses, getPrograms, getTokenMints, getUsers } from "../../utils/fixtures";
import { Keypair } from "@solana/web3.js";
import { createMarketPDA, createMarketTokenMintPDA, createMarketVaultPDA, createRolesPDA, getMarketSignPDA } from "../../utils/data";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

describe("exchange: market", () => {
    const { market } = getPrograms();
    const { signer0 } = getUsers();
    const { dataStoreAddress } = getAddresses();
    const { dataStore } = getPrograms();
    const { BTC_TOKEN_MINT, SOL_TOKEN_MINT } = getTokenMints();

    const [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);

    const indexTokenMint = Keypair.generate().publicKey;
    const longTokenMint = BTC_TOKEN_MINT;
    const shortTokenMint = SOL_TOKEN_MINT;

    it("create market", async () => {
        const [marketTokenMint] = createMarketTokenMintPDA(dataStoreAddress, indexTokenMint, longTokenMint, shortTokenMint);
        const [longToken] = createMarketVaultPDA(dataStoreAddress, longTokenMint, marketTokenMint);
        const [shortToken] = createMarketVaultPDA(dataStoreAddress, shortTokenMint, marketTokenMint);
        const [marketSign] = getMarketSignPDA();
        await market.methods.createMarket(indexTokenMint).accounts({
            authority: signer0.publicKey,
            onlyMarketKeeper: roles,
            dataStore: dataStoreAddress,
            market: createMarketPDA(dataStoreAddress, marketTokenMint)[0],
            marketTokenMint,
            longTokenMint,
            shortTokenMint,
            longToken,
            shortToken,
            marketSign,
            dataStoreProgram: dataStore.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        }).signers([signer0]).rpc();
    });
});
