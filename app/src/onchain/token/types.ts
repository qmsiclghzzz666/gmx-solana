import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

export interface Token {
  symbol: string,
  address: PublicKey,
  decimals: number,
  feedAddress?: PublicKey,
}

export interface TokenPrices {
  minPrice: BN,
  maxPrice: BN,
}

export type TokenData = Token & {
  prices: TokenPrices,
  balance?: BN,
  totalSupply?: BN,
};

export interface Prices {
  [feedAddress: string]: TokenPrices,
}

export interface Tokens {
  [address: string]: TokenData,
}
