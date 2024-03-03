import * as anchor from "@coral-xyz/anchor";
import { RoleStore } from "../target/types/role_store";
import { keyToSeed } from "./seed";

export const roleStore = anchor.workspace.RoleStore as anchor.Program<RoleStore>;

export const ROLE_STORE_SEED = anchor.utils.bytes.utf8.encode("role_store");
export const ROLE_SEED = anchor.utils.bytes.utf8.encode("role");

export const ROLE_ADMIN = "ROLE_ADMIN";
export const CONTROLLER = "CONTROLLER";

export const createRoleStorePDA = (key: string) => anchor.web3.PublicKey.findProgramAddressSync([
    ROLE_STORE_SEED,
    keyToSeed(key),
], roleStore.programId);
export const createRolePDA = (store: anchor.web3.PublicKey, roleName: string, authority: anchor.web3.PublicKey) => anchor.web3.PublicKey.findProgramAddressSync([
    ROLE_SEED,
    store.toBytes(),
    anchor.utils.bytes.utf8.encode(roleName),
    authority.toBytes(),
], roleStore.programId);

export const createRoleAdminPDA = (store: anchor.web3.PublicKey, authority: anchor.web3.PublicKey) => createRolePDA(store, ROLE_ADMIN, authority);
export const createControllerPDA = (store: anchor.web3.PublicKey, authority: anchor.web3.PublicKey) => createRolePDA(store, CONTROLLER, authority);

export const initializeRoleStore = async (provider: anchor.AnchorProvider, key: string, controller: anchor.web3.PublicKey) => {
    const [store] = createRoleStorePDA(key);
    const [onlyRoleAdmin] = createRoleAdminPDA(store, provider.wallet.publicKey);

    // Initialize the RoleStore with the first admin to be the wallet.
    try {
        const tx = await roleStore.methods.initialize(key).accounts({
            authority: provider.wallet.publicKey,
            store,
            roleAdmin: onlyRoleAdmin,
        }).rpc();
        console.log(`Initialized a new role store account ${store.toBase58()} in tx: ${tx}`);
    } catch (error) {
        console.warn("Failed to initialize a role store, maybe it has been initialized", error);
    }
    const [onlyController0] = createControllerPDA(store, controller);
    // Grant CONTROLLER role to the `controller`.
    try {
        const tx = await roleStore.methods.grantRole(CONTROLLER).accounts({
            authority: provider.wallet.publicKey,
            store,
            onlyRoleAdmin,
            roleAuthority: controller,
            role: onlyController0,
        }).rpc();
        console.log(`Granted CONTROLLER role to ${controller} in tx ${tx}`);
    } catch (error) {
        console.warn(`Failed to grant CONTROLLER role to ${controller}`, error);
    }
    return store;
};
