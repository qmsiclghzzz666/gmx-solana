import * as anchor from "@coral-xyz/anchor";
import { Oracle } from "../target/types/oracle";

export const oracle = anchor.workspace.Oracle as anchor.Program<Oracle>;
