import { convertToUsd, expandDecimals } from "@/utils/number";
import { TokenData, Tokens } from "../token";
import { MarketInfo, MarketState, MarketTokens } from "./types";
import { toBN } from "gmsol";
import { BN_ZERO, ONE_USD } from "@/config/constants";
import { Address, BN, translateAddress } from "@coral-xyz/anchor";
import { convertToTokenAmount, getMidPrice } from "../token/utils";
import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";

export function usdToMarketTokenAmount(poolValue: BN, marketToken: TokenData, usdValue: BN) {
  const supply = marketToken.totalSupply!;
  // const poolValue = marketInfo.poolValueMax!;
  // if the supply and poolValue is zero, use 1 USD as the token price
  if (supply.isZero() && poolValue.isZero()) {
    return convertToTokenAmount(usdValue, marketToken.decimals, ONE_USD)!;
  }

  // if the supply is zero and the poolValue is more than zero,
  // then include the poolValue for the amount of tokens minted so that
  // the market token price after mint would be 1 USD
  if (supply.isZero() && poolValue.gt(BN_ZERO)) {
    return convertToTokenAmount(usdValue.add(poolValue), marketToken.decimals, ONE_USD)!;
  }

  if (poolValue.isZero()) {
    return BN_ZERO;
  }

  return supply.mul(usdValue).div(poolValue);
}

export function getSellableMarketToken(marketInfo: MarketInfo, marketToken: TokenData) {
  const { longToken, shortToken, longPoolAmount, shortPoolAmount } = marketInfo;
  const longPoolUsd = convertToUsd(longPoolAmount, longToken.decimals, getMidPrice(longToken.prices))!;
  const shortPoolUsd = convertToUsd(shortPoolAmount, shortToken.decimals, getMidPrice(shortToken.prices))!;
  // const longCollateralLiquidityUsd = getAvailableUsdLiquidityForCollateral(marketInfo, true);
  // const shortCollateralLiquidityUsd = getAvailableUsdLiquidityForCollateral(marketInfo, false);
  const longCollateralLiquidityUsd = longPoolUsd;
  const shortCollateralLiquidityUsd = shortPoolUsd;

  const factor = expandDecimals(toBN(1), 8);

  if (
    longPoolUsd.isZero() ||
    shortPoolUsd.isZero() ||
    longCollateralLiquidityUsd.isZero() ||
    shortCollateralLiquidityUsd.isZero()
  ) {
    return {
      maxLongSellableUsd: BN_ZERO,
      maxShortSellableUsd: BN_ZERO,
      total: BN_ZERO,
    };
  }

  const ratio = longPoolUsd.mul(factor).div(shortPoolUsd);
  let maxLongSellableUsd: BN;
  let maxShortSellableUsd: BN;

  if (shortCollateralLiquidityUsd.mul(ratio).div(factor).lte(longCollateralLiquidityUsd)) {
    maxLongSellableUsd = shortCollateralLiquidityUsd.mul(ratio).div(factor);
    maxShortSellableUsd = shortCollateralLiquidityUsd;
  } else {
    maxLongSellableUsd = longCollateralLiquidityUsd;
    maxShortSellableUsd = longCollateralLiquidityUsd.div(ratio).mul(factor);
  }

  const poolValue = longPoolUsd.add(shortPoolUsd);
  const maxLongSellableAmount = usdToMarketTokenAmount(poolValue, marketToken, maxLongSellableUsd);
  const maxShortSellableAmount = usdToMarketTokenAmount(poolValue, marketToken, maxShortSellableUsd);

  return {
    maxLongSellableUsd,
    maxShortSellableUsd,
    maxLongSellableAmount,
    maxShortSellableAmount,
    totalUsd: maxLongSellableUsd.add(maxShortSellableUsd),
    totalAmount: maxLongSellableAmount.add(maxShortSellableAmount),
  };
}

export function getPoolUsdWithoutPnl(
  marketInfo: MarketTokens & MarketState,
  isLong: boolean,
  priceType: "minPrice" | "maxPrice" | "midPrice"
) {
  const poolAmount = isLong ? marketInfo.longPoolAmount : marketInfo.shortPoolAmount;
  const token = isLong ? marketInfo.longToken : marketInfo.shortToken;

  let price: BN;

  if (priceType === "minPrice") {
    price = token.prices?.minPrice;
  } else if (priceType === "maxPrice") {
    price = token.prices?.maxPrice;
  } else {
    price = getMidPrice(token.prices);
  }

  return convertToUsd(poolAmount, token.decimals, price)!;
}

/**
 * Apart from usual cases, returns `long` for single token backed markets.
 */
export function getTokenPoolType(marketInfo: MarketInfo, tokenAddress: Address): "long" | "short" | undefined {
  const translated = translateAddress(tokenAddress);

  const { longToken, shortToken } = marketInfo;

  if (longToken.address.equals(shortToken.address) && translated.equals(longToken.address)) {
    return "long";
  }

  if (translated.equals(longToken.address) || (translated.equals(NATIVE_TOKEN_ADDRESS) && longToken.isWrapped)) {
    return "long";
  }

  if (translated.equals(shortToken.address) || (translated.equals(NATIVE_TOKEN_ADDRESS) && shortToken.isWrapped)) {
    return "short";
  }

  return undefined;
}

export function getTotalGmInfo(tokensData?: Tokens) {
  const defaultResult = {
    balance: BN_ZERO,
    balanceUsd: BN_ZERO,
  };

  if (!tokensData) {
    return defaultResult;
  }

  const tokens = Object.values(tokensData).filter((token) => token.symbol === "GM");

  return tokens.reduce((acc, token) => {
    const balanceUsd = convertToUsd(token.balance ?? BN_ZERO, token.decimals, token.prices.minPrice);
    acc.balance = acc.balance.add(token.balance || BN_ZERO);
    acc.balanceUsd = acc.balanceUsd.add(balanceUsd || BN_ZERO);
    return acc;
  }, defaultResult);
}
