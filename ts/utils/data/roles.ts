import { PublicKey, Signer } from "@solana/web3.js";
import { dataStore } from "./program";
import { anchor } from "../endpoint";

// Roles seed.
export const ROLES_SEED = anchor.utils.bytes.utf8.encode("roles");

export const createRolesPDA = (store: PublicKey, authority: PublicKey) => PublicKey.findProgramAddressSync([
    ROLES_SEED,
    store.toBytes(),
    authority.toBytes(),
], dataStore.programId);

