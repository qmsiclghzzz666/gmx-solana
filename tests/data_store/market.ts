import { Keypair, Transaction } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { createMarketPDA, createMarketTokenMintPDA, createMarketVaultPDA, createRolesPDA, getMarketSignPDA } from "../../utils/data";
import { createAssociatedTokenAccountInstruction, createTransferInstruction, getAssociatedTokenAddress } from "@solana/spl-token";

describe("data store: Market", () => {
    const { dataStore } = getPrograms();
    const { signer0, user0 } = getUsers();

    const { dataStoreAddress } = getAddresses();
    const provider = getProvider();

    const [roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);

    const indexToken = Keypair.generate().publicKey;
    const longToken = Keypair.generate().publicKey;
    const shortToken = Keypair.generate().publicKey;
    const marketToken = Keypair.generate().publicKey;
    const [marketPDA] = createMarketPDA(dataStoreAddress, marketToken);

    it("init and remove a market", async () => {
        await dataStore.methods.initializeMarket(marketToken, indexToken, longToken, shortToken).accounts({
            authority: signer0.publicKey,
            onlyMarketKeeper: roles,
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
            onlyMarketKeeper: roles,
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
        const [marketSign] = getMarketSignPDA();

        await dataStore.methods.initializeMarketToken(indexToken, longToken, shortToken).accounts({
            store: dataStoreAddress,
            authority: signer0.publicKey,
            onlyMarketKeeper: roles,
            marketTokenMint,
            marketSign,
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
            onlyController: roles,
            marketTokenMint,
            marketSign,
            to: userTokenAccount,
        }).signers([signer0]).rpc();

        const [marketVault] = createMarketVaultPDA(dataStoreAddress, marketTokenMint);
        await dataStore.methods.initializeMarketVault(null).accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyMarketKeeper: roles,
            mint: marketTokenMint,
            vault: marketVault,
            marketSign,
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
            onlyController: roles,
            store: dataStoreAddress,
            marketSign,
            marketVault,
            to: userTokenAccount,
        }).signers([signer0]).rpc();

    });
});
