import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { DataStore } from "../target/types/data_store";
import { RoleStore } from "../target/types/role_store";
import { PublicKey, Keypair } from '@solana/web3.js';
import { sha256 } from "js-sha256";

import { user } from "./role_store";

chai.use(chaiAsPromised);
const expect = chai.expect;

const toSeed = (key: string) => anchor.utils.bytes.hex.decode(sha256(key));

const addressSeed = anchor.utils.bytes.utf8.encode("address");
const membershipSeed = anchor.utils.bytes.utf8.encode("membership");
const roleController = anchor.utils.bytes.utf8.encode("CONTROLLER");
const roleAdmin = anchor.utils.bytes.utf8.encode("ROLE_ADMIN");

describe("data store", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const dataStore = anchor.workspace.DataStore as Program<DataStore>;
    const roleStore = anchor.workspace.RoleStore as Program<RoleStore>;


    const [onlyAdmin] = PublicKey.findProgramAddressSync([
        membershipSeed,
        roleAdmin,
        provider.wallet.publicKey.toBytes(),
    ], roleStore.programId);

    const [onlyController] = PublicKey.findProgramAddressSync([
        membershipSeed,
        roleController,
        provider.wallet.publicKey.toBytes(),
    ], roleStore.programId);

    const [helloMembership] = PublicKey.findProgramAddressSync([
        membershipSeed,
        anchor.utils.bytes.utf8.encode("HELLO"),
        user.publicKey.toBytes(),
    ], roleStore.programId);

    const key = Keypair.generate().publicKey;
    const fooAddressKey = `PRICE_FEED:${key}`;
    const fooAddress = Keypair.generate().publicKey;
    const fooSeed = toSeed(fooAddressKey);
    const [fooAddressPDA] = PublicKey.findProgramAddressSync([
        addressSeed,
        fooSeed,
    ], dataStore.programId);

    it("set and get address", async () => {
        await roleStore.methods.grantRole("CONTROLLER").accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin,
            member: provider.wallet.publicKey,
            membership: onlyController,
        }).rpc();
        await dataStore.methods.setAddress(fooAddressKey, fooAddress).accounts({
            authority: provider.wallet.publicKey,
            onlyController,
            address: fooAddressPDA,
        }).rpc();
        const saved = await dataStore.methods.getAddress(fooAddressKey).accounts({
            address: fooAddressPDA,
        }).view() as PublicKey;
        expect(saved).to.eql(fooAddress);
    });

    it("only controller", async () => {
        await expect(dataStore.methods.setAddress(fooAddressKey, fooAddress).accounts({
            authority: user.publicKey,
            onlyController: helloMembership,
            address: fooAddressPDA,
        }).rpc()).to.be.rejected;
    });
});
