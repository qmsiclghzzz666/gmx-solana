import { convertToUsd, expandDecimals } from "@/utils/number";
import { TokenData, TokenOption, Tokens } from "../token";
import { MarketInfo, MarketInfos, MarketState, MarketTokens } from "./types";
import { toBN } from "gmsol";
import { BN_ZERO, ONE_USD } from "@/config/constants";
import { Address, BN, translateAddress } from "@coral-xyz/anchor";
import { convertToTokenAmount, getMidPrice } from "../token/utils";
import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";
import { TradeType } from "../trade";
import Graph from "graphology";

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

export function getAvailableUsdLiquidityForPosition(marketInfo: MarketInfo, isLong: boolean) {
  if (marketInfo.isSpotOnly) {
    return BN_ZERO;
  }

  // const maxReservedUsd = getMaxReservedUsd(marketInfo, isLong);
  // const reservedUsd = getReservedUsd(marketInfo, isLong);

  // const maxOpenInterest = getMaxOpenInterestUsd(marketInfo, isLong);
  // const currentOpenInterest = getOpenInterestUsd(marketInfo, isLong);

  // const availableLiquidityBasedOnMaxReserve = maxReservedUsd.sub(reservedUsd);
  // const availableLiquidityBasedOnMaxOpenInterest = maxOpenInterest.sub(currentOpenInterest);

  // const result = availableLiquidityBasedOnMaxReserve.lt(availableLiquidityBasedOnMaxOpenInterest)
  //   ? availableLiquidityBasedOnMaxReserve
  //   : availableLiquidityBasedOnMaxOpenInterest;

  // return result.lt(0) ? BigNumber.from(0) : result;
  return getPoolUsdWithoutPnl(marketInfo, isLong, "midPrice");
}

export type PreferredTradeTypePickStrategy = TradeType | "largestPosition";

export function chooseSuitableMarket({
  indexTokenAddress,
  maxLongLiquidityPool,
  maxShortLiquidityPool,
  isSwap,
  // positionsInfo,
  preferredTradeType,
}: {
  indexTokenAddress: string;
  maxLongLiquidityPool: TokenOption;
  maxShortLiquidityPool: TokenOption;
  isSwap?: boolean;
  // positionsInfo?: PositionsInfoData;
  preferredTradeType: PreferredTradeTypePickStrategy;
}):
  | { indexTokenAddress: string; marketTokenAddress?: string; tradeType: TradeType; collateralTokenAddress?: string }
  | undefined {
  if (isSwap) {
    return {
      indexTokenAddress,
      tradeType: TradeType.Swap,
    };
  }

  if (preferredTradeType === "largestPosition") {
    // if (positionsInfo) {
    //   let largestLongPosition: PositionInfo | undefined = getLargestRelatedExistingPosition({
    //     positionsInfo,
    //     isLong: true,
    //     indexTokenAddress,
    //   });

    //   let largestShortPosition: PositionInfo | undefined = getLargestRelatedExistingPosition({
    //     positionsInfo,
    //     isLong: false,
    //     indexTokenAddress,
    //   });

    //   if (!largestLongPosition && !largestShortPosition) {
    //     return {
    //       indexTokenAddress,
    //       marketTokenAddress: maxLongLiquidityPool.marketTokenAddress,
    //       tradeType: TradeType.Long,
    //     };
    //   }

    //   let largestPosition: PositionInfo;
    //   if (largestLongPosition && largestShortPosition) {
    //     largestPosition = largestLongPosition.sizeInUsd.gt(largestShortPosition.sizeInUsd)
    //       ? largestLongPosition
    //       : largestShortPosition;
    //   } else {
    //     largestPosition = largestLongPosition! || largestShortPosition!;
    //   }
    //   const largestPositionTradeType = largestPosition?.isLong ? TradeType.Long : TradeType.Short;

    //   return {
    //     indexTokenAddress,
    //     marketTokenAddress: largestPosition.marketInfo.marketTokenAddress,
    //     tradeType: largestPositionTradeType,
    //     collateralTokenAddress: largestPosition.collateralTokenAddress,
    //   };
    // }

    return {
      indexTokenAddress,
      marketTokenAddress: maxLongLiquidityPool.marketTokenAddress,
      tradeType: TradeType.Long,
    };
  }

  if (preferredTradeType === TradeType.Long) {
    // const largestLongPosition =
    //   positionsInfo &&
    //   getLargestRelatedExistingPosition({
    //     positionsInfo,
    //     isLong: true,
    //     indexTokenAddress,
    //   });

    // const marketTokenAddress =
    //   largestLongPosition?.marketInfo.marketTokenAddress ?? maxLongLiquidityPool.marketTokenAddress;

    const marketTokenAddress = maxLongLiquidityPool.marketTokenAddress;

    return {
      indexTokenAddress,
      marketTokenAddress: marketTokenAddress,
      tradeType: TradeType.Long,
      collateralTokenAddress: undefined,
    };
  } else {
    // const largestShortPosition =
    //   positionsInfo &&
    //   getLargestRelatedExistingPosition({
    //     positionsInfo,
    //     isLong: false,
    //     indexTokenAddress,
    //   });

    // const marketTokenAddress =
    //   largestShortPosition?.marketInfo.marketTokenAddress ?? maxShortLiquidityPool.marketTokenAddress;

    const marketTokenAddress = maxShortLiquidityPool.marketTokenAddress;

    return {
      indexTokenAddress,
      marketTokenAddress,
      tradeType: TradeType.Short,
      collateralTokenAddress: undefined,
    };
  }
}

export function makeMarketGraph(marketInfos: MarketInfos) {
  const graph = new Graph({
    multi: true,
    type: "directed",
  });
  for (const address in marketInfos) {
    const market = marketInfos[address];
    const longAddress = market.longTokenAddress.toBase58();
    const shortAddress = market.shortTokenAddress.toBase58();
    const longCapacity = getPoolUsdWithoutPnl(market, true, "minPrice");
    const shortCapacity = getPoolUsdWithoutPnl(market, false, "minPrice");
    graph.mergeNode(longAddress);
    graph.mergeNode(shortAddress);
    graph.addEdgeWithKey([address, "long"], longAddress, shortAddress, {
      capacity: shortCapacity,
      fee: longCapacity.gt(shortCapacity) ? 0 : 1,
    });
    graph.addEdgeWithKey([address, "short"], shortAddress, longAddress, {
      capacity: longCapacity,
      fee: shortCapacity.gt(longCapacity) ? 0 : 1,
    });
  }
  return graph;
}

export function edgeNameToMarketTokenAddress(edgeName: string) {
  try {
    return edgeName.split(',')[0];
  } catch (error) {
    throw Error("Not a valid edge name");
  }
}
