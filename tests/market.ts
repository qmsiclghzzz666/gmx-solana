import * as anchor from "@coral-xyz/anchor";
import { expect, getAddresses, getPrograms, getUsers } from "../utils/fixtures";
import { Market } from "../target/types/market";
import { Keypair } from "@solana/web3.js";
import { createMarketKeeperPDA } from "../utils/role";
import { createMarketPDA } from "../utils/data";

describe("market", () => {
    const market = anchor.workspace.Market as anchor.Program<Market>;
    const { signer0 } = getUsers();
    const { roleStoreAddress, dataStoreAddress } = getAddresses();
    const { dataStore } = getPrograms();

    const indexToken = Keypair.generate().publicKey;
    const longToken = Keypair.generate().publicKey;
    const shortToken = Keypair.generate().publicKey;

    it("create market", async () => {
        await market.methods.createMarket(
            indexToken,
            longToken,
            shortToken,
            Keypair.generate().publicKey,
        ).accounts({
            authority: signer0.publicKey,
            onlyMarketKeeper: createMarketKeeperPDA(roleStoreAddress, signer0.publicKey)[0],
            dataStore: dataStoreAddress,
            market: createMarketPDA(dataStoreAddress, indexToken, longToken, shortToken)[0],
            dataStoreProgram: dataStore.programId,
        }).signers([signer0]).rpc();
    });
});
