import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { DataStore } from "../target/types/data_store";
import { RoleStore } from "../target/types/role_store";

import { user } from "./role_store";

chai.use(chaiAsPromised);
const expect = chai.expect;

const membershipSeed = anchor.utils.bytes.utf8.encode("membership");
const roleAdmin = anchor.utils.bytes.utf8.encode("ROLE_ADMIN");

describe("data store", () => {
    const provider = anchor.AnchorProvider.env();
    // Configure the client to use the local cluster.
    anchor.setProvider(provider);

    const program = anchor.workspace.DataStore as Program<DataStore>;
    const roleStore = anchor.workspace.RoleStore as Program<RoleStore>;

    const [adminMembershipPDA,] = anchor.web3.PublicKey.findProgramAddressSync([
        membershipSeed,
        roleAdmin,
        provider.wallet.publicKey.toBytes(),
    ], roleStore.programId);

    const [userMembershipPDA,] = anchor.web3.PublicKey.findProgramAddressSync([
        membershipSeed,
        anchor.utils.bytes.utf8.encode("HELLO"),
        user.publicKey.toBytes(),
    ], roleStore.programId);

    const [userAdminPDA] = anchor.web3.PublicKey.findProgramAddressSync([
        membershipSeed,
        roleAdmin,
        user.publicKey.toBytes(),
    ], roleStore.programId);

    it("Initialize data store", async () => {
        await program.methods.initialize().accounts(
            {
                authority: provider.wallet.publicKey,
                membership: adminMembershipPDA,
            }
        ).rpc();
    });

    it("Initialize data store should fail", async () => {
        await expect(program.methods.initialize().accounts(
            {
                authority: user.publicKey,
                membership: userMembershipPDA,
            }
        ).signers([user]).rpc()).to.be.rejected;
    });

    it("Initialize data store with user", async () => {
        await roleStore.methods.grantRole("ROLE_ADMIN").accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin: adminMembershipPDA,
            member: user.publicKey,
            membership: userAdminPDA,
        });
        console.log("signing with", user.publicKey);
        await expect(program.methods.initialize().accounts(
            {
                authority: user.publicKey,
                membership: userAdminPDA,
            }
        ).signers([user]).rpc()).to.be.ok;
    });
});
