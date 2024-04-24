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
  balance?: BN | null,
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

export type TokenInfo = Token & {
  maxPrice?: BN;
  minPrice?: BN;
};

export interface InfoTokens {
  [address: string]: TokenInfo,
}

export interface TokenBalances {
  [address: string]: BN | null,
}
