import { Keypair } from '@solana/web3.js';

import { expect, getAddresses, getPrograms, getUsers } from "../../utils/fixtures";
import { createMarketKeeperPDA } from "../../utils/role";
import { createMarketPDA } from "../../utils/data";

describe("data store: Market", () => {
    const { dataStore } = getPrograms();
    const { signer0 } = getUsers();

    const { roleStoreAddress, dataStoreAddress } = getAddresses();
    const [onlyMarketKeeper] = createMarketKeeperPDA(roleStoreAddress, signer0.publicKey);

    const indexToken = Keypair.generate().publicKey;
    const longToken = Keypair.generate().publicKey;
    const shortToken = Keypair.generate().publicKey;
    const marketToken = Keypair.generate().publicKey;
    const [marketPDA] = createMarketPDA(dataStoreAddress, marketToken);

    it("init and remove a market", async () => {
        await dataStore.methods.initializeMarket(marketToken, indexToken, longToken, shortToken).accounts({
            authority: signer0.publicKey,
            onlyMarketKeeper,
            store: dataStoreAddress,
            market: marketPDA,
        }).signers([signer0]).rpc();
        {
            const market = await dataStore.account.market.fetch(marketPDA);
            expect(market.indexTokenMint).eql(indexToken);
            expect(market.longTokenMint).eql(longToken);
            expect(market.shortTokenMint).eql(shortToken);
            expect(market.marketTokenMint).eql(marketToken);
        }
        await dataStore.methods.removeMarket().accounts({
            authority: signer0.publicKey,
            onlyMarketKeeper,
            store: dataStoreAddress,
            market: marketPDA,
        }).signers([signer0]).rpc();
        {
            const market = await dataStore.account.market.getAccountInfo(marketPDA);
            expect(market).to.be.null;
        }
    });
});
