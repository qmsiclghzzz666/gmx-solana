import * as anchor from "@coral-xyz/anchor";
import { sha256 } from "js-sha256";

export const keyToSeed = (key: string) => anchor.utils.bytes.hex.decode(sha256(key));
