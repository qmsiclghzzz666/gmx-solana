import * as anchor from "@coral-xyz/anchor";
import { RoleStore } from "../target/types/role_store";

export const roleStore = anchor.workspace.RoleStore as anchor.Program<RoleStore>;

export const MEMBERSHIP_SEED = anchor.utils.bytes.utf8.encode("membership");

export const ROLE_ADMIN = "ROLE_ADMIN";
export const CONTROLLER = "CONTROLLER";

export const createMembershipPDA = (roleKey: string, authority: anchor.web3.PublicKey) => anchor.web3.PublicKey.findProgramAddressSync([
    MEMBERSHIP_SEED,
    anchor.utils.bytes.utf8.encode(roleKey),
    authority.toBytes(),
], roleStore.programId);

export const createRoleAdminPDA = (authority: anchor.web3.PublicKey) => createMembershipPDA(ROLE_ADMIN, authority);
export const createControllerPDA = (authority: anchor.web3.PublicKey) => createMembershipPDA(CONTROLLER, authority);

export const initializeRoleStore = async (provider: anchor.AnchorProvider, controller: anchor.web3.PublicKey) => {
    const [onlyAdmin] = createRoleAdminPDA(provider.wallet.publicKey);
    // Initialize the RoleStore with the first admin to be the wallet.
    try {
        const tx = await roleStore.methods.initialize().accounts({
            admin: provider.wallet.publicKey,
            adminMembership: onlyAdmin,
        }).rpc();
        console.log("Initialized the RoleStore program in tx:", tx);
    } catch (error) {
        console.warn("Failed to initialize the RoleStore program, maybe it has been initialized", error);
    }

    // Grant CONTROLLER role to the `controller`.
    const tx = await roleStore.methods.grantRole(CONTROLLER).accounts({
        authority: provider.wallet.publicKey,
        onlyAdmin: onlyAdmin,
        member: controller,
        membership: createControllerPDA(controller)[0],
    }).rpc();
    console.log("Granted CONTROLLER to the given `controller` in tx:", tx);
};
