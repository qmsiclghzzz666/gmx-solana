import { PublicKey, Keypair } from '@solana/web3.js';

import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { createMarketKeeperPDA } from "../../utils/role";
import { createMarketPDA } from "../../utils/data";
import { AnchorError } from '@coral-xyz/anchor';

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
            expect(market.indexToken).eql(indexToken);
            expect(market.longToken).eql(longToken);
            expect(market.shortToken).eql(shortToken);
            expect(market.marketToken).eql(marketToken);
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
