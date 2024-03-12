import * as anchor from "@coral-xyz/anchor";
import { Market } from "../target/types/market";
import { PublicKey } from "@solana/web3.js";

export const market = anchor.workspace.Market as anchor.Program<Market>;

export const MARKET_TOKEN_SEED = anchor.utils.bytes.utf8.encode("market_token");

export const createMarketTokenPDA = (indexToken: PublicKey, longToken: PublicKey, shortToken: PublicKey) => PublicKey.findProgramAddressSync([
    MARKET_TOKEN_SEED,
    indexToken.toBytes(),
    longToken.toBytes(),
    shortToken.toBytes(),
], market.programId);

export const getMarketTokenAuthority = () => PublicKey.findProgramAddressSync([], market.programId);
