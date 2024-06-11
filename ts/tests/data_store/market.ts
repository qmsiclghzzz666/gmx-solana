import { Keypair, PublicKey, Transaction } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { createMarketPDA, createMarketTokenMintPDA, createMarketVaultPDA, createRolesPDA } from "../../utils/data";
import { createAssociatedTokenAccountInstruction, createTransferInstruction, getAssociatedTokenAddress } from "@solana/spl-token";
import { invokeUpdateMarketConfig } from '../../utils/exchange';

describe("data store: Market", () => {
    const { dataStore } = getPrograms();
    const { signer0, user0 } = getUsers();

    const provider = getProvider();

    const indexToken = Keypair.generate().publicKey;
    const longToken = Keypair.generate().publicKey;
    const shortToken = Keypair.generate().publicKey;
    const marketToken = Keypair.generate().publicKey;

    let dataStoreAddress: PublicKey;
    let roles: PublicKey;
    let marketPDA: PublicKey;
    before(async () => {
        ({ dataStoreAddress } = await getAddresses());
        [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
        [marketPDA] = createMarketPDA(dataStoreAddress, marketToken);
    });

    it("create, update and remove a market", async () => {
        await dataStore.methods.initializeMarket(marketToken, indexToken, longToken, shortToken).accountsPartial({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            market: marketPDA,
        }).signers([signer0]).rpc();
        {
            const market = await dataStore.account.market.fetch(marketPDA);
            expect(market.meta.indexTokenMint).eql(indexToken);
            expect(market.meta.longTokenMint).eql(longToken);
            expect(market.meta.shortTokenMint).eql(shortToken);
            expect(market.meta.marketTokenMint).eql(marketToken);
        }
        {
            await expect(invokeUpdateMarketConfig(dataStore, {
                authority: user0,
                store: dataStoreAddress,
                marketToken,
                key: "swap_fee_receiver_factor",
                value: 99000000000000000001n,
            })).rejectedWith(Error, "Permission denied");
        }
        {
            const tx = await invokeUpdateMarketConfig(dataStore, {
                authority: signer0,
                store: dataStoreAddress,
                marketToken,
                key: "swap_fee_receiver_factor",
                value: 37000000000000000001n,
            });
            console.log(`market config updated at ${tx}`);
            const value = await dataStore.methods.getMarketConfig("swap_fee_receiver_factor").accounts({ market: marketPDA }).view();
            expect((new BN("37000000000000000001")).eq(value));
        }
        await dataStore.methods.removeMarket().accountsPartial({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            market: marketPDA,
        }).signers([signer0]).rpc();
        {
            const market = await dataStore.account.market.getAccountInfo(marketPDA);
            expect(market).to.be.null;
        }
    });

    it("perform basic token operations", async () => {
        const [marketTokenMint] = createMarketTokenMintPDA(dataStoreAddress, indexToken, longToken, shortToken);
        await dataStore.methods.initializeMarketToken(indexToken, longToken, shortToken).accountsPartial({
            store: dataStoreAddress,
            authority: signer0.publicKey,
            marketTokenMint,
        }).signers([signer0]).rpc();

        const userTokenAccount = await getAssociatedTokenAddress(marketTokenMint, user0.publicKey);
        await provider.sendAndConfirm(new Transaction().add(createAssociatedTokenAccountInstruction(
            provider.publicKey,
            userTokenAccount,
            user0.publicKey,
            marketTokenMint,
        )));

        await dataStore.methods.mintMarketTokenTo(new BN("100000000").mul(new BN(100))).accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            marketTokenMint,
            to: userTokenAccount,
        }).signers([signer0]).rpc();

        const [marketVault] = createMarketVaultPDA(dataStoreAddress, marketTokenMint);
        await dataStore.methods.initializeMarketVault(null).accountsPartial({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            mint: marketTokenMint,
            vault: marketVault,
        }).signers([signer0]).rpc();

        await provider.sendAndConfirm(new Transaction().add(createTransferInstruction(
            userTokenAccount,
            marketVault,
            user0.publicKey,
            100000000 * 50,
        )), [
            user0,
        ]);

        await dataStore.methods.marketVaultTransferOut(new BN("100000000").mul(new BN(11))).accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            marketVault,
            to: userTokenAccount,
        }).signers([signer0]).rpc();

    });
});
