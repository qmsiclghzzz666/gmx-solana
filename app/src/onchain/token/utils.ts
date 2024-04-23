import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";
import { TokenPrices, Tokens } from "./types";
import { Address, BN } from "@coral-xyz/anchor";
import { expandDecimals } from "@/utils/number";
import { BN_ZERO } from "@/config/constants";

export function getTokenData(tokensData?: Tokens, address?: Address, convertTo?: "wrapped" | "native") {
  const addressStr = address?.toString();
  if (!addressStr || !tokensData?.[addressStr]) {
    return undefined;
  }

  const token = tokensData[addressStr];

  if (convertTo === "wrapped" && token.isNative && token.wrappedAddress) {
    return tokensData[token.wrappedAddress.toBase58()];
  }

  if (convertTo === "native" && token.isWrapped) {
    return tokensData[NATIVE_TOKEN_ADDRESS.toBase58()];
  }

  return token;
}

export function convertToTokenAmount(
  usd: BN | undefined,
  tokenDecimals: number | undefined,
  price: BN | undefined
) {
  if (!usd || typeof tokenDecimals !== "number" || !price?.gt(BN_ZERO)) {
    return undefined;
  }

  return usd.mul(expandDecimals(new BN(1), tokenDecimals)).div(price);
}

export function getMidPrice(prices: TokenPrices) {
  return prices.minPrice.add(prices.maxPrice).div(new BN(2));
}
