import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RoleStore } from "../target/types/role_store";
import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';

chai.use(chaiAsPromised);
const expect = chai.expect;

const membershipSeed = anchor.utils.bytes.utf8.encode("membership");
const roleAdmin = anchor.utils.bytes.utf8.encode("ROLE_ADMIN");
export const user = anchor.web3.Keypair.generate();

describe("role store", () => {
    const provider = anchor.AnchorProvider.env();
    // Configure the client to use the local cluster.
    anchor.setProvider(provider);

    const program = anchor.workspace.RoleStore as Program<RoleStore>;

    const [adminMembershipPDA,] = anchor.web3.PublicKey.findProgramAddressSync([
        membershipSeed,
        roleAdmin,
        provider.wallet.publicKey.toBytes(),
    ], program.programId);

    const [userMembershipPDA,] = anchor.web3.PublicKey.findProgramAddressSync([
        membershipSeed,
        anchor.utils.bytes.utf8.encode("HELLO"),
        user.publicKey.toBytes(),
    ], program.programId);

    const user2 = anchor.web3.Keypair.generate();
    const [user2MembershipPDA,] = anchor.web3.PublicKey.findProgramAddressSync([
        membershipSeed,
        anchor.utils.bytes.utf8.encode("HELLO"),
        user2.publicKey.toBytes(),
    ], program.programId);

    it("Initialize role store", async () => {
        // Add your test here.
        const tx = await program.methods.initialize().accounts(
            {
                admin: provider.wallet.publicKey,
                adminMembership: adminMembershipPDA,
            }
        ).rpc();
        const adminRole = (await program.account.membership.fetch(adminMembershipPDA)).role;
        expect(
            adminRole
        ).to.equal("ROLE_ADMIN");
    });

    it("Grant role", async () => {
        const tx = await program.methods.grantRole("HELLO").accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin: adminMembershipPDA,
            member: user.publicKey,
            membership: userMembershipPDA
        }).rpc();
        const userMember = await program.account.membership.fetch(userMembershipPDA);
        expect(userMember.role).to.equals("HELLO");
    });

    it("Grant role should failed", async () => {
        await expect(program.methods.grantRole("HELLO").accounts({
            authority: user.publicKey,
            onlyAdmin: userMembershipPDA,
            member: user2.publicKey,
            membership: user2MembershipPDA
        }).signers([user]).rpc()).to.be.rejected;
    });

    it("Revoke role", async () => {
        await program.methods.grantRole("HELLO").accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin: adminMembershipPDA,
            member: user2.publicKey,
            membership: user2MembershipPDA
        }).rpc();
        expect(await program.account.membership.getAccountInfo(user2MembershipPDA)).to.be.not.null;
        await program.methods.revokeRole("HELLO").accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin: adminMembershipPDA,
            member: user2.publicKey,
            membership: user2MembershipPDA,
        }).rpc();
        expect(await program.account.membership.getAccountInfo(user2MembershipPDA)).to.be.null;
    });
});
