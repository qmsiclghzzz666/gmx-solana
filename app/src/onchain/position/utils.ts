import { BN } from "@coral-xyz/anchor";
import { Token, TokenData, getIsEquivalentTokens } from "../token";
import { applyFactor, convertToUsd, expandDecimals } from "@/utils/number";
import { BN_ONE, BN_ZERO } from "@/config/constants";
import { MarketInfo } from "../market";
import { BASIS_POINTS_DIVISOR } from "@/config/factors";
import { formatAmount, formatUsd } from "@/components/MarketsList/utils";

export function getEntryPrice(p: { sizeInUsd: BN; sizeInTokens: BN; indexToken: Token }) {
  const { sizeInUsd, sizeInTokens, indexToken } = p;

  if (!sizeInTokens.gt(BN_ZERO)) {
    return undefined;
  }

  return sizeInUsd.div(sizeInTokens).mul(expandDecimals(new BN(1), indexToken.decimals));
}

export function getPositionPendingFeesUsd(p: { pendingFundingFeesUsd: BN; pendingBorrowingFeesUsd: BN }) {
  const { pendingFundingFeesUsd, pendingBorrowingFeesUsd } = p;

  return pendingBorrowingFeesUsd.add(pendingFundingFeesUsd);
}

export function getPositionNetValue(p: {
  collateralUsd: BN;
  pendingFundingFeesUsd: BN;
  pendingBorrowingFeesUsd: BN;
  pnl: BN;
  closingFeeUsd: BN;
  uiFeeUsd: BN;
}) {
  const { pnl, closingFeeUsd, collateralUsd, uiFeeUsd } = p;

  const pendingFeesUsd = getPositionPendingFeesUsd(p);

  return collateralUsd.sub(pendingFeesUsd).sub(closingFeeUsd).sub(uiFeeUsd).add(pnl);
}

export function getPositionValueUsd(p: { indexToken: Token; sizeInTokens: BN; markPrice: BN }) {
  const { indexToken, sizeInTokens, markPrice } = p;

  return convertToUsd(sizeInTokens, indexToken.decimals, markPrice)!;
}

export function getPositionPnlUsd(p: {
  marketInfo: MarketInfo;
  sizeInUsd: BN;
  sizeInTokens: BN;
  markPrice: BN;
  isLong: boolean;
}) {
  const { marketInfo, sizeInUsd, sizeInTokens, markPrice, isLong } = p;

  const positionValueUsd = getPositionValueUsd({ indexToken: marketInfo.indexToken, sizeInTokens, markPrice });

  const totalPnl = isLong ? positionValueUsd.sub(sizeInUsd) : sizeInUsd.sub(positionValueUsd);

  if (totalPnl.lte(BN_ZERO)) {
    return totalPnl;
  }

  // const poolPnl = isLong ? p.marketInfo.pnlLongMax : p.marketInfo.pnlShortMax;
  // const poolUsd = getPoolUsdWithoutPnl(marketInfo, isLong, "minPrice");

  // const cappedPnl = getCappedPoolPnl({
  //   marketInfo,
  //   poolUsd,
  //   isLong,
  //   maximize: true,
  // });

  // const WEI_PRECISION = expandDecimals(1, 18);

  // if (!cappedPnl.eq(poolPnl) && cappedPnl.gt(0) && poolPnl.gt(0)) {
  //   totalPnl = totalPnl.mul(cappedPnl.div(WEI_PRECISION)).div(poolPnl.div(WEI_PRECISION));
  // }

  return totalPnl;
}

export function getLeverage(p: {
  sizeInUsd: BN;
  collateralUsd: BN;
  pnl: BN | undefined;
  pendingFundingFeesUsd: BN;
  pendingBorrowingFeesUsd: BN;
}) {
  const { pnl, sizeInUsd, collateralUsd, pendingBorrowingFeesUsd, pendingFundingFeesUsd } = p;

  const totalPendingFeesUsd = getPositionPendingFeesUsd({ pendingFundingFeesUsd, pendingBorrowingFeesUsd });

  const remainingCollateralUsd = collateralUsd.add(pnl || BN_ZERO).sub(totalPendingFeesUsd);

  if (remainingCollateralUsd.lte(BN_ZERO)) {
    return undefined;
  }

  return sizeInUsd.muln(BASIS_POINTS_DIVISOR).div(remainingCollateralUsd);
}

export function formatLeverage(leverage?: BN) {
  if (!leverage) return undefined;

  return `${formatAmount(leverage, 4, 2)}x`;
}

export function getLiquidationPrice(p: {
  sizeInUsd: BN;
  sizeInTokens: BN;
  collateralAmount: BN;
  collateralUsd: BN;
  collateralToken: TokenData;
  marketInfo: MarketInfo;
  pendingFundingFeesUsd: BN;
  pendingBorrowingFeesUsd: BN;
  minCollateralUsd: BN;
  isLong: boolean;
  useMaxPriceImpact?: boolean;
  // userReferralInfo: UserReferralInfo | undefined;
}) {
  const {
    sizeInUsd,
    sizeInTokens,
    collateralUsd,
    collateralAmount,
    marketInfo,
    collateralToken,
    pendingFundingFeesUsd,
    pendingBorrowingFeesUsd,
    minCollateralUsd,
    isLong,
    // userReferralInfo,
    // useMaxPriceImpact,
  } = p;

  if (!sizeInUsd.gt(BN_ZERO) || !sizeInTokens.gt(BN_ZERO)) {
    return undefined;
  }

  const { indexToken } = marketInfo;

  // const closingFeeUsd = getPositionFee(marketInfo, sizeInUsd, false, userReferralInfo).positionFeeUsd;
  const closingFeeUsd = BN_ZERO;
  const totalPendingFeesUsd = getPositionPendingFeesUsd({ pendingFundingFeesUsd, pendingBorrowingFeesUsd });
  const totalFeesUsd = totalPendingFeesUsd.add(closingFeeUsd);

  // const maxNegativePriceImpactUsd = applyFactor(sizeInUsd, marketInfo.maxPositionImpactFactorForLiquidations).mul(-1);

  // let priceImpactDeltaUsd: BN = BN_ZERO;

  // if (useMaxPriceImpact) {
  //   priceImpactDeltaUsd = maxNegativePriceImpactUsd;
  // } else {
  //   priceImpactDeltaUsd = getPriceImpactForPosition(marketInfo, sizeInUsd.mul(-1), isLong, { fallbackToZero: true });

  //   if (priceImpactDeltaUsd.lt(maxNegativePriceImpactUsd)) {
  //     priceImpactDeltaUsd = maxNegativePriceImpactUsd;
  //   }

  //   // Ignore positive price impact
  //   if (priceImpactDeltaUsd.gt(0)) {
  //     priceImpactDeltaUsd = BN.from(0);
  //   }
  // }

  let liquidationCollateralUsd = applyFactor(sizeInUsd, marketInfo.minCollateralFactor);
  if (liquidationCollateralUsd.lt(minCollateralUsd)) {
    liquidationCollateralUsd = minCollateralUsd;
  }

  let liquidationPrice: BN;

  if (getIsEquivalentTokens(collateralToken, indexToken)) {
    if (isLong) {
      const denominator = sizeInTokens.add(collateralAmount);

      if (denominator.eq(BN_ZERO)) {
        return undefined;
      }

      liquidationPrice = sizeInUsd
        .add(liquidationCollateralUsd)
        // .sub(priceImpactDeltaUsd)
        .add(totalFeesUsd)
        .div(denominator)
        .mul(expandDecimals(BN_ONE, indexToken.decimals));
    } else {
      const denominator = sizeInTokens.sub(collateralAmount);
      if (denominator.eq(BN_ZERO)) {
        return undefined;
      }

      liquidationPrice = sizeInUsd
        .sub(liquidationCollateralUsd)
        // .add(priceImpactDeltaUsd)
        .sub(totalFeesUsd)
        .div(denominator)
        .mul(expandDecimals(BN_ONE, indexToken.decimals));
    }
  } else {
    if (sizeInTokens.eq(BN_ZERO)) {
      return undefined;
    }

    const remainingCollateralUsd = collateralUsd
      // .add(priceImpactDeltaUsd)
      .sub(totalPendingFeesUsd)
      .sub(closingFeeUsd);

    if (isLong) {
      liquidationPrice = liquidationCollateralUsd
        .sub(remainingCollateralUsd)
        .add(sizeInUsd)
        .div(sizeInTokens)
        .mul(expandDecimals(BN_ONE, indexToken.decimals));
    } else {
      liquidationPrice = liquidationCollateralUsd
        .sub(remainingCollateralUsd)
        .sub(sizeInUsd)
        .div(sizeInTokens.muln(-1))
        .mul(expandDecimals(BN_ONE, indexToken.decimals));
    }
  }

  if (liquidationPrice.lte(BN_ZERO)) {
    return undefined;
  }

  return liquidationPrice;
}

export function formatLiquidationPrice(liquidationPrice?: BN, opts: { displayDecimals?: number } = {}) {
  if (!liquidationPrice || liquidationPrice.lte(BN_ZERO)) {
    return "NA";
  }

  return formatUsd(liquidationPrice, { ...opts, maxThreshold: "1000000" });
}
