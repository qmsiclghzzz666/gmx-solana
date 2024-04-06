import { AnchorError } from "@coral-xyz/anchor";
import { createRolesPDA } from "../../utils/data";
import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";
import { PublicKey } from "@solana/web3.js";

describe("data store: Roles", () => {
    const provider = getProvider();
    const { signer0 } = getUsers();
    const { dataStore } = getPrograms();

    const otherRole = "OTHER";

    let dataStoreAddress: PublicKey;
    let signer0Roles: PublicKey;
    let providerRoles: PublicKey;
    before(async () => {
        ({ dataStoreAddress } = await getAddresses());
        [signer0Roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
        [providerRoles] = createRolesPDA(dataStoreAddress, provider.publicKey);

        await dataStore.methods.enableRole(otherRole).accounts({
            authority: provider.publicKey,
            store: dataStoreAddress,
            onlyAdmin: providerRoles,
        }).rpc();
    });

    after(async () => {
        await dataStore.methods.disableRole(otherRole).accounts({
            authority: provider.publicKey,
            store: dataStoreAddress,
            onlyAdmin: providerRoles,
        }).rpc();
    });


    it("check admin success", async () => {
        const isAdmin: boolean = await dataStore.methods.hasAdmin(provider.publicKey).accounts({
            store: dataStoreAddress,
            roles: providerRoles,
        }).view();
        expect(isAdmin).true;
    });

    it("check admin failure", async () => {
        const isAdmin = await dataStore.methods.hasAdmin(signer0.publicKey).accounts({
            store: dataStoreAddress,
            roles: signer0Roles,
        }).view();
        expect(isAdmin).false;
    });

    it("should fail to enable role without admin role", async () => {
        await expect(dataStore.methods.enableRole("FOO").accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyAdmin: signer0Roles,
        }).signers([signer0]).rpc()).to.rejectedWith(AnchorError, "Not an admin");
    });

    it("should fail to disable role without admin role", async () => {
        await expect(dataStore.methods.disableRole("CONTROLLER").accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyAdmin: signer0Roles,
        }).signers([signer0]).rpc()).to.rejectedWith(AnchorError, "Not an admin");
    });

    it("enable and disable a new role", async () => {
        await expect(dataStore.methods.enableRole("FOO").accounts({
            authority: provider.publicKey,
            store: dataStoreAddress,
            onlyAdmin: signer0Roles,
        }).rpc());
        await expect(dataStore.methods.disableRole("FOO").accounts({
            authority: provider.publicKey,
            store: dataStoreAddress,
            onlyAdmin: providerRoles,
        }).rpc());
    });

    it("should fail to enable or disable role without correct roles", async () => {
        await expect(dataStore.methods.enableRole("FOO").accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyAdmin: providerRoles,
        }).signers([signer0]).rpc()).to.rejectedWith(AnchorError, "Permission denied");

        await expect(dataStore.methods.enableRole("FOO").accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyAdmin: providerRoles,
        }).signers([signer0]).rpc()).to.rejectedWith(AnchorError, "Permission denied");
    });

    it("grant, check and revoke a role to user", async () => {
        await dataStore.methods.grantRole(signer0.publicKey, otherRole).accounts({
            authority: provider.publicKey,
            store: dataStoreAddress,
            onlyAdmin: providerRoles,
            userRoles: signer0Roles,
        }).rpc();

        {
            const hasRole = await dataStore.methods.hasRole(signer0.publicKey, otherRole).accounts({
                store: dataStoreAddress,
                roles: signer0Roles,
            }).view();
            expect(hasRole).true;
        }

        await dataStore.methods.revokeRole(signer0.publicKey, otherRole).accounts({
            authority: provider.publicKey,
            store: dataStoreAddress,
            onlyAdmin: providerRoles,
            userRoles: signer0Roles,
        }).rpc();

        {
            const hasRole = await dataStore.methods.hasRole(signer0.publicKey, otherRole).accounts({
                store: dataStoreAddress,
                roles: signer0Roles,
            }).view();
            expect(hasRole).false;
        }

        await expect(dataStore.methods.grantRole(signer0.publicKey, otherRole).accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyAdmin: signer0Roles,
            userRoles: signer0Roles,
        }).signers([signer0]).rpc()).to.rejectedWith(AnchorError, "Not an admin");

        await expect(dataStore.methods.revokeRole(signer0.publicKey, otherRole).accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyAdmin: signer0Roles,
            userRoles: signer0Roles,
        }).signers([signer0]).rpc()).to.rejectedWith(AnchorError, "Not an admin");
    });
});
