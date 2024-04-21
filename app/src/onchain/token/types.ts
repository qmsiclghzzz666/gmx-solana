import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

export interface Token {
  symbol: string,
  address: PublicKey,
  decimals: number,
  feedAddress?: PublicKey,
  isNative?: boolean,
  wrappedAddress?: PublicKey,
  isWrapped?: boolean,
}

export interface TokenPrices {
  minPrice: BN,
  maxPrice: BN,
}

export type TokenData = Token & TokenMetadata & {
  prices: TokenPrices,
  balance?: BN,
};

export interface TokenMetadata {
  decimals: number,
  totalSupply?: BN,
}

export interface TokenMetadatas {
  [address: string]: TokenMetadata,
}

export interface Prices {
  [feedAddress: string]: TokenPrices,
}

export interface Tokens {
  [address: string]: TokenData,
}
