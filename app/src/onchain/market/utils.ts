import { convertToUsd, expandDecimals } from "@/utils/number";
import { TokenData } from "../token";
import { MarketInfo } from "./types";
import { toBN } from "gmsol";
import { BN_ZERO, ONE_USD } from "@/config/constants";
import { BN } from "@coral-xyz/anchor";
import { convertToTokenAmount, getMidPrice } from "../token/utils";

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
  const longPoolUsd = convertToUsd(longPoolAmount, longToken.decimals, longToken.prices.maxPrice)!;
  const shortPoolUsd = convertToUsd(shortPoolAmount, shortToken.decimals, shortToken.prices.maxPrice)!;
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
  marketInfo: MarketInfo,
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