import { createRolesPDA } from "../../utils/data";
import { expect, getAddresses, getPrograms, getProvider, getUsers } from "../../utils/fixtures";

describe("data store: Roles", () => {
    const provider = getProvider();
    const { signer0 } = getUsers();
    const { dataStoreAddress } = getAddresses();
    const { dataStore } = getPrograms();

    it("check admin success", async () => {
        const isAdmin: boolean = await dataStore.methods.checkAdmin(provider.publicKey).accounts({
            store: dataStoreAddress,
            roles: createRolesPDA(dataStoreAddress, provider.publicKey)[0],
        }).view();
        expect(isAdmin).true;
    });

    it("check admin failure", async () => {
        const isAdmin = await dataStore.methods.checkAdmin(signer0.publicKey).accounts({
            store: dataStoreAddress,
            roles: createRolesPDA(dataStoreAddress, signer0.publicKey)[0],
        }).view();
        expect(isAdmin).false;
    });
});
