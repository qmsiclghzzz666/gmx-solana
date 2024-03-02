import * as anchor from "@coral-xyz/anchor";
import { getProvider, getPrograms, expect, getUsers } from "../utils/fixtures";
import { createMembershipPDA, createRoleAdminPDA } from "../utils/role";

const membershipSeed = anchor.utils.bytes.utf8.encode("membership");
const roleAdmin = anchor.utils.bytes.utf8.encode("ROLE_ADMIN");
export const user = anchor.web3.Keypair.generate();

describe("role store", () => {
    const provider = getProvider();
    const {
        roleStore,
    } = getPrograms();
    const {
        signer0,
    } = getUsers();

    const helloRole = "HELLO";
    const [onlyAdmin] = createRoleAdminPDA(provider.wallet.publicKey);
    const [helloMembership] = createMembershipPDA(helloRole, user.publicKey);

    it(`grant a role to a user`, async () => {
        await roleStore.methods.grantRole(helloRole).accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin,
            member: user.publicKey,
            membership: helloMembership,
        }).rpc();
        expect((await roleStore.account.membership.fetch(helloMembership)).role).to.equals(helloRole);
    });

    it("should fail to grant a role without admin role", async () => {
        await expect(roleStore.methods.grantRole("OTHER").accounts({
            authority: signer0.publicKey,
            onlyAdmin: createRoleAdminPDA(signer0.publicKey)[0],
            member: user.publicKey,
            membership: createMembershipPDA("OTHER", user.publicKey)[0],
        }).signers([signer0]).rpc()).to.be.rejected;
    });

    it("should fail to revoke a role without admin role", async () => {
        await expect(roleStore.methods.revokeRole(helloRole).accounts({
            authority: signer0.publicKey,
            onlyAdmin: createRoleAdminPDA(signer0.publicKey)[0],
            member: user.publicKey,
            membership: helloMembership,
        }).signers([signer0]).rpc()).to.be.rejected;
    });

    it("grant a role and then revoke it", async () => {
        const role = "OTHER";
        const [membership] = createMembershipPDA(role, user.publicKey);
        await roleStore.methods.grantRole(role).accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin,
            member: user.publicKey,
            membership,
        }).rpc();
        expect(await roleStore.account.membership.getAccountInfo(membership)).to.be.not.null;
        expect((await roleStore.account.membership.fetch(membership)).role).to.equals(role);
        await roleStore.methods.revokeRole(role).accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin,
            member: user.publicKey,
            membership,
        }).rpc();
        expect(await roleStore.account.membership.getAccountInfo(membership)).to.be.null;
    });
});
