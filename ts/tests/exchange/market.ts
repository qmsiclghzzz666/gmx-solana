import { expect, getAddresses, getTokenMints, getUsers } from "../../utils/fixtures";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createMarketTokenMintPDA, createRolesPDA, storeProgram, invokePushToTokenMapSynthetic } from "../../utils/data";
import { createMarket } from "../../utils/exchange";
import { AnchorError } from "@coral-xyz/anchor";
import { findMarketPDA } from "gmsol";

describe("exchange: Market", () => {
    const { signer0, user0 } = getUsers();
    const { BTC_TOKEN_MINT, SOL_TOKEN_MINT } = getTokenMints();

    const indexTokenMint = Keypair.generate().publicKey;
    const longTokenMint = BTC_TOKEN_MINT;
    const shortTokenMint = SOL_TOKEN_MINT;

    let dataStoreAddress: PublicKey;
    let roles: PublicKey;
    before(async () => {
        ({ dataStoreAddress } = await getAddresses());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
        const tokenMap = (await storeProgram.account.store.fetch(dataStoreAddress)).tokenMap;
        await invokePushToTokenMapSynthetic(storeProgram, {
            authority: signer0,
            store: dataStoreAddress,
            tokenMap,
            name: "fake",
            token: indexTokenMint,
            tokenDecimals: 6,
            heartbeatDuration: 33,
            precision: 4,
            feeds: {}
        });
    });

    it("create market", async () => {
        const [marketTokenMint] = createMarketTokenMintPDA(dataStoreAddress, indexTokenMint, longTokenMint, shortTokenMint);
        const [market] = findMarketPDA(dataStoreAddress, marketTokenMint);
        await createMarket(signer0, "test", dataStoreAddress, indexTokenMint, longTokenMint, shortTokenMint);
        // Only MARKET_KEEPER can toggle market.
        {
            await expect(storeProgram.methods.toggleMarket(false).accounts({
                authority: user0.publicKey,
                market,
            }).signers([user0]).rpc()).rejectedWith(Error, "Permission denied");
        }
        {
            const beforeFlag = (await storeProgram.account.market.fetch(market)).flag;
            await storeProgram.methods.toggleMarket(false).accounts({
                authority: signer0.publicKey,
                market,
            }).signers([signer0]).rpc();
            const afterFlag = (await storeProgram.account.market.fetch(market)).flag;
            expect(beforeFlag).not.eql(afterFlag);
        }
    });

    it("only market keeper can create market", async () => {
        (await expect(createMarket(user0, "test", dataStoreAddress, indexTokenMint, longTokenMint, longTokenMint))).rejectedWith(AnchorError, "Permission denied");
    });
});
