import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RoleStore } from "../target/types/role_store";
import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';

chai.use(chaiAsPromised);
const expect = chai.expect;

const membershipSeed = anchor.utils.bytes.utf8.encode("membership");

describe("role store", () => {
    const provider = anchor.AnchorProvider.env();
    // Configure the client to use the local cluster.
    anchor.setProvider(provider);

    const program = anchor.workspace.RoleStore as Program<RoleStore>;

    const [adminMembershipPDA,] = anchor.web3.PublicKey.findProgramAddressSync([
        membershipSeed,
        provider.wallet.publicKey.toBytes(),
    ], program.programId);

    const user = anchor.web3.Keypair.generate();
    const [userMembershipPDA,] = anchor.web3.PublicKey.findProgramAddressSync([
        membershipSeed,
        user.publicKey.toBytes(),
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
            permission: {
                authority: {
                    signer: provider.wallet.publicKey,
                    membership: adminMembershipPDA,
                }
            },
            member: user.publicKey,
            membership: userMembershipPDA
        }).rpc();
        const userMember = await program.account.membership.fetch(userMembershipPDA);
        expect(userMember.role).to.equals("HELLO");
    });

    it("Grant role should failed", async () => {
        const user2 = anchor.web3.Keypair.generate();
        const [user2MembershipPDA,] = anchor.web3.PublicKey.findProgramAddressSync([
            membershipSeed,
            user2.publicKey.toBytes(),
        ], program.programId);
        await expect(program.methods.grantRole("HELLO").accounts({
            permission: {
                authority: {
                    signer: user.publicKey,
                    membership: userMembershipPDA,
                }
            },
            member: user2.publicKey,
            membership: user2MembershipPDA
        }).signers([user]).rpc()).to.be.rejectedWith(anchor.AnchorError, /Error Message: Invalid role/);
    });
});
