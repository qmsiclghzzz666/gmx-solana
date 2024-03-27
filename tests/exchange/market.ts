import { getAddresses, getPrograms, getTokenMints, getUsers } from "../../utils/fixtures";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createMarketPDA, createMarketTokenMintPDA, createMarketVaultPDA, createRolesPDA, getMarketSignPDA } from "../../utils/data";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

describe("exchange: market", () => {
    const { exchange } = getPrograms();
    const { signer0 } = getUsers();
    const { dataStore } = getPrograms();
    const { BTC_TOKEN_MINT, SOL_TOKEN_MINT } = getTokenMints();

    const indexTokenMint = Keypair.generate().publicKey;
    const longTokenMint = BTC_TOKEN_MINT;
    const shortTokenMint = SOL_TOKEN_MINT;

    let dataStoreAddress: PublicKey;
    let roles: PublicKey;
    before(async () => {
        ({ dataStoreAddress } = await getAddresses());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
    });

    it("create market", async () => {
        const [marketTokenMint] = createMarketTokenMintPDA(dataStoreAddress, indexTokenMint, longTokenMint, shortTokenMint);
        const [marketSign] = getMarketSignPDA();
        await exchange.methods.createMarket(indexTokenMint).accounts({
            authority: signer0.publicKey,
            onlyMarketKeeper: roles,
            dataStore: dataStoreAddress,
            market: createMarketPDA(dataStoreAddress, marketTokenMint)[0],
            marketTokenMint,
            longTokenMint,
            shortTokenMint,
            marketSign,
            dataStoreProgram: dataStore.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        }).signers([signer0]).rpc();
    });
});
