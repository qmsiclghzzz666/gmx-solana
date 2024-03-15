import { AnchorError } from "@coral-xyz/anchor";
import { createRolesPDA } from "../../utils/data";
import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";

describe("data store: Roles", () => {
    const provider = getProvider();
    const { signer0 } = getUsers();
    const { dataStoreAddress } = getAddresses();
    const { dataStore } = getPrograms();

    const [signer0Roles] = createRolesPDA(dataStoreAddress, signer0.publicKey);
    const [providerRoles] = createRolesPDA(dataStoreAddress, provider.publicKey);

    it("check admin success", async () => {
        const isAdmin: boolean = await dataStore.methods.checkAdmin(provider.publicKey).accounts({
            store: dataStoreAddress,
            roles: providerRoles,
        }).view();
        expect(isAdmin).true;
    });

    it("check admin failure", async () => {
        const isAdmin = await dataStore.methods.checkAdmin(signer0.publicKey).accounts({
            store: dataStoreAddress,
            roles: signer0Roles,
        }).view();
        expect(isAdmin).false;
    });

    it("should fail to enable role without admin role", async () => {
        await expect(dataStore.methods.enableRole("HELLO").accounts({
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
        await expect(dataStore.methods.enableRole("HELLO").accounts({
            authority: provider.publicKey,
            store: dataStoreAddress,
            onlyAdmin: signer0Roles,
        }).rpc());
        await expect(dataStore.methods.disableRole("HELLO").accounts({
            authority: provider.publicKey,
            store: dataStoreAddress,
            onlyAdmin: providerRoles,
        }).rpc());
    });

    it("should fail to enable or disable role without correct roles", async () => {
        await expect(dataStore.methods.enableRole("HELLO").accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyAdmin: providerRoles,
        }).signers([signer0]).rpc()).to.rejectedWith(AnchorError, "Permission denied");

        await expect(dataStore.methods.enableRole("HELLO").accounts({
            authority: signer0.publicKey,
            store: dataStoreAddress,
            onlyAdmin: providerRoles,
        }).signers([signer0]).rpc()).to.rejectedWith(AnchorError, "Permission denied");
    });
});
