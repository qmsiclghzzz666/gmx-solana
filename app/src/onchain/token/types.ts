import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

export interface Token {
  symbol: string,
  address: PublicKey,
  decimals: number,
  priceDecimals?: number,
  feedAddress?: string,
  isStable?: boolean,
  isNative?: boolean,
  wrappedAddress?: PublicKey,
  isWrapped?: boolean,
  isWrappedNative?: boolean,
  isSynthetic?: boolean,
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
  maxMintable?: BN,
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

export interface TokenBalances {
  [address: string]: BN | null,
}

export type TokenOption = {
  maxLongLiquidity: BN;
  maxShortLiquidity: BN;
  marketTokenAddress: string;
  indexTokenAddress: string;
};
