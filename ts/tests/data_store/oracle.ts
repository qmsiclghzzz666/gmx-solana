import * as anchor from "@coral-xyz/anchor";
import { getAddresses, getPrograms, getProvider, getUsers, expect } from "../../utils/fixtures";
import { createRolesPDA, dataStore } from "../../utils/data";
import { BTC_FEED, BTC_FEED_PYTH, BTC_TOKEN_MINT, SOL_FEED, SOL_FEED_PYTH, SOL_TOKEN_MINT, USDC_FEED, USDC_FEED_PYTH } from "../../utils/token";
import { PublicKey } from "@solana/web3.js";
import { PYTH_ID } from "../../utils/external";

describe("data store: oracle", () => {
    const provider = getProvider();

    const { signer0 } = getUsers();

    const mockFeedAccount = anchor.web3.Keypair.generate();

    let dataStoreAddress: PublicKey;
    let oracleAddress: PublicKey;
    let roles: PublicKey;
    let fakeTokenMint: PublicKey;
    let usdGTokenMint: PublicKey;
    before(async () => {
        ({ dataStoreAddress, oracleAddress, fakeTokenMint, usdGTokenMint } = await getAddresses());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
    });

    it("set price from feed and then clear", async () => {
        const tokenMap = (await dataStore.account.store.fetch(dataStoreAddress)).tokenMap;
        await dataStore.methods.setPricesFromPriceFeed([
            BTC_TOKEN_MINT,
            SOL_TOKEN_MINT,
            fakeTokenMint,
            usdGTokenMint,
        ]).accountsPartial({
            store: dataStoreAddress,
            authority: signer0.publicKey,
            tokenMap,
            oracle: oracleAddress,
            priceProvider: PYTH_ID,
        }).remainingAccounts([
            {
                pubkey: BTC_FEED_PYTH,
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: SOL_FEED_PYTH,
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: BTC_FEED_PYTH,
                isSigner: false,
                isWritable: false,
            },
            {
                pubkey: USDC_FEED_PYTH,
                isSigner: false,
                isWritable: false,
            },
        ]).signers([signer0]).rpc();
        const setData = await dataStore.account.oracle.fetch(oracleAddress);
        // console.log(setData.primary.prices);
        expect(setData.primary.prices.length).to.equal(4);

        await dataStore.methods.clearAllPrices().accountsPartial({
            store: dataStoreAddress,
            authority: signer0.publicKey,
            oracle: oracleAddress,
        }).signers([signer0]).rpc();
        const clearedData = await dataStore.account.oracle.fetch(oracleAddress);
        expect(clearedData.primary.prices.length).to.equal(0);
        expect(clearedData.primary.tokens.length).to.equal(0);
    });
});
