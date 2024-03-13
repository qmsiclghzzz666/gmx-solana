import * as anchor from "@coral-xyz/anchor";
import { getProvider, getPrograms, expect, getUsers, getKeys } from "../utils/fixtures";
import { createRolePDA, createRoleAdminPDA, createRoleStorePDA, ROLE_ADMIN, createControllerPDA } from "../utils/role";

const user = anchor.web3.Keypair.generate();

describe("role store", () => {
    const provider = getProvider();
    const { roleStore } = getPrograms();
    const { signer0 } = getUsers();
    const { roleStoreKey } = getKeys();

    const helloRoleName = "HELLO";
    const [store] = createRoleStorePDA(roleStoreKey);
    const [onlyRoleAdmin] = createRoleAdminPDA(store, provider.wallet.publicKey);
    const [helloRole] = createRolePDA(store, helloRoleName, user.publicKey);

    const anotherStoreKey = anchor.web3.Keypair.generate().publicKey.toBase58();
    const [anotherStore] = createRoleStorePDA(anotherStoreKey);
    const [anotherAdmin] = createRoleAdminPDA(anotherStore, signer0.publicKey);

    before(async () => {
        await roleStore.methods.initialize(anotherStoreKey).accounts({
            authority: signer0.publicKey,
            store: anotherStore,
            roleAdmin: anotherAdmin,
        }).signers([signer0]).rpc();
    });

    it(`grant a role to a user`, async () => {
        await roleStore.methods.grantRole(helloRoleName).accounts({
            authority: provider.wallet.publicKey,
            store,
            onlyRoleAdmin,
            roleAuthority: user.publicKey,
            role: helloRole,
        }).rpc();
        const fetchedRole = await roleStore.account.role.fetch(helloRole);
        expect(fetchedRole.role).to.equal(helloRoleName);
        expect(fetchedRole.store).to.eql(store);
    });

    it("should fail to grant a role without ROLE_ADMIN role", async () => {
        await expect(roleStore.methods.grantRole("OTHER").accounts({
            authority: signer0.publicKey,
            store,
            onlyRoleAdmin: createControllerPDA(store, signer0.publicKey)[0],
            roleAuthority: user.publicKey,
            role: createRolePDA(store, "OTHER", user.publicKey)[0],
        }).signers([signer0]).rpc()).to.be.rejectedWith(anchor.AnchorError, "Permission denied");
    });

    it("should fail to revoke a role without ROLE_ADMIN role", async () => {
        await expect(roleStore.methods.revokeRole().accounts({
            authority: signer0.publicKey,
            store,
            onlyRoleAdmin: createControllerPDA(store, signer0.publicKey)[0],
            role: helloRole,
        }).signers([signer0]).rpc()).to.be.rejectedWith(anchor.AnchorError, "Permission denied");
    });

    it("should fail to grant a role with other store", async () => {
        await expect(roleStore.methods.grantRole("OTHER").accounts({
            authority: signer0.publicKey,
            store,
            onlyRoleAdmin: anotherAdmin,
            roleAuthority: user.publicKey,
            role: createRolePDA(store, "OTHER", user.publicKey)[0],
        }).signers([signer0]).rpc()).to.be.rejectedWith(anchor.AnchorError, "Mismatched store");
    });

    it("should fail to revoke a role with other store", async () => {
        await expect(roleStore.methods.revokeRole().accounts({
            authority: signer0.publicKey,
            store,
            onlyRoleAdmin: anotherAdmin,
            role: helloRole,
        }).signers([signer0]).rpc()).to.be.rejectedWith(anchor.AnchorError, "Mismatched store");
    });

    it("tests grant-revoke multiple times", async () => {
        const roleName = "OTHER";
        const [role] = createRolePDA(store, roleName, user.publicKey);
        await roleStore.methods.grantRole(roleName).accounts({
            authority: provider.wallet.publicKey,
            store,
            onlyRoleAdmin,
            roleAuthority: user.publicKey,
            role,
        }).rpc();
        expect(await roleStore.account.role.getAccountInfo(role)).to.be.not.null;
        expect((await roleStore.account.role.fetch(role)).role).to.equals(roleName);
        // Cannot grant again without revoking it first.
        await expect(roleStore.methods.grantRole(roleName).accounts({
            authority: provider.wallet.publicKey,
            store,
            onlyRoleAdmin,
            roleAuthority: user.publicKey,
            role,
        }).rpc()).to.be.rejected;
        await roleStore.methods.revokeRole().accounts({
            authority: provider.wallet.publicKey,
            store,
            onlyRoleAdmin,
            role,
        }).rpc();
        expect(await roleStore.account.role.getAccountInfo(role)).to.be.null;
        await roleStore.methods.grantRole(roleName).accounts({
            authority: provider.wallet.publicKey,
            store,
            onlyRoleAdmin,
            roleAuthority: user.publicKey,
            role,
        }).rpc();
        expect(await roleStore.account.role.getAccountInfo(role)).to.be.not.null;
        expect((await roleStore.account.role.fetch(role)).role).to.equal(roleName);
    });

    // it("cannot revoke ROLE_ADMIN role with the only ROLE_ADMIN role member", async () => {
    //     await expect(roleStore.methods.revokeRole().accounts({
    //         authority: provider.wallet.publicKey,
    //         store,
    //         onlyRoleAdmin,
    //         role: onlyRoleAdmin,
    //     }).rpc()).to.be.rejectedWith(anchor.AnchorError, "At least one admin per store");
    // });

    it("can revoke ROLE_ADMIN role when there are other ROLE_ADMIN role members", async () => {
        const anotherAdmin = anchor.web3.Keypair.generate();
        const [role] = createRoleAdminPDA(store, anotherAdmin.publicKey);
        await roleStore.methods.grantRole(ROLE_ADMIN).accounts({
            authority: provider.wallet.publicKey,
            store,
            onlyRoleAdmin,
            roleAuthority: anotherAdmin.publicKey,
            role,
        }).rpc();
        await expect(roleStore.methods.revokeRole().accounts({
            authority: anotherAdmin.publicKey,
            store,
            onlyRoleAdmin: role,
            role,
        }).rpc()).to.be.ok;
    });
});
