import * as anchor from "@coral-xyz/anchor";
import { Market } from "../target/types/market";
import { PublicKey } from "@solana/web3.js";

export const market = anchor.workspace.Market as anchor.Program<Market>;

export const MARKET_TOKEN_MINT_SEED = anchor.utils.bytes.utf8.encode("market_token_mint");
export const LONG_TOKEN_SEED = anchor.utils.bytes.utf8.encode("long_token");
export const SHORT_TOKEN_SEED = anchor.utils.bytes.utf8.encode("short_token");

export const createMarketTokenMintPDA = (
    dataStore: PublicKey,
    indexTokenMint: PublicKey,
    longTokenMint: PublicKey,
    shortTokenMint: PublicKey,
) => PublicKey.findProgramAddressSync([
    MARKET_TOKEN_MINT_SEED,
    dataStore.toBytes(),
    indexTokenMint.toBytes(),
    longTokenMint.toBytes(),
    shortTokenMint.toBytes(),
], market.programId);

export const createLongTokenPDA = (marketTokenMint: PublicKey) => PublicKey.findProgramAddressSync([
    LONG_TOKEN_SEED,
    marketTokenMint.toBytes(),
], market.programId);

export const createShortTokenPDA = (marketTokenMint: PublicKey) => PublicKey.findProgramAddressSync([
    SHORT_TOKEN_SEED,
    marketTokenMint.toBytes(),
], market.programId);

export const getMarketTokenAuthority = () => PublicKey.findProgramAddressSync([], market.programId);
