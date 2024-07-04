import { GmsolStore } from "../../../target/types/gmsol_store";
import { anchor } from "../endpoint";

export const storeProgram = anchor.workspace.GmsolStore as anchor.Program<GmsolStore>;
