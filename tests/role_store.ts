import * as anchor from "@coral-xyz/anchor";
import { getProvider, getPrograms, expect, getUsers } from "../utils/fixtures";
import { createMembershipPDA, createRoleAdminPDA } from "../utils/role";

const user = anchor.web3.Keypair.generate();

describe("role store", () => {
    const provider = getProvider();
    const {
        roleStore,
    } = getPrograms();
    const {
        user0,
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
            authority: user0.publicKey,
            onlyAdmin: createRoleAdminPDA(user0.publicKey)[0],
            member: user.publicKey,
            membership: createMembershipPDA("OTHER", user.publicKey)[0],
        }).signers([user0]).rpc()).to.be.rejected;
    });

    it("should fail to revoke a role without admin role", async () => {
        await expect(roleStore.methods.revokeRole(helloRole).accounts({
            authority: user0.publicKey,
            onlyAdmin: createRoleAdminPDA(user0.publicKey)[0],
            member: user.publicKey,
            membership: helloMembership,
        }).signers([user0]).rpc()).to.be.rejected;
    });

    it("tests grant-revoke multiple times", async () => {
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
        // Cannot grant again without revoking it first.
        await expect(roleStore.methods.grantRole(role).accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin,
            member: user.publicKey,
            membership,
        }).rpc()).to.be.rejected;
        await roleStore.methods.revokeRole(role).accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin,
            member: user.publicKey,
            membership,
        }).rpc();
        expect(await roleStore.account.membership.getAccountInfo(membership)).to.be.null;
        await roleStore.methods.grantRole(role).accounts({
            authority: provider.wallet.publicKey,
            onlyAdmin,
            member: user.publicKey,
            membership,
        }).rpc();
        expect(await roleStore.account.membership.getAccountInfo(membership)).to.be.not.null;
        expect((await roleStore.account.membership.fetch(membership)).role).to.equals(role);
    });
});
