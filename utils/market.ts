import { workspace, Program } from "@coral-xyz/anchor";
import { Market } from "../target/types/market";

export const market = workspace.Market as Program<Market>;
