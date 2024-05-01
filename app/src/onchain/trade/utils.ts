import { PositionInfo } from "../position";
import { TradeFlags, TradeMode, TradeParams, TradeType } from "./types";

export const createTradeFlags = (tradeType: TradeType, tradeMode: TradeMode): TradeFlags => {
  const isLong = tradeType === TradeType.Long;
  const isShort = tradeType === TradeType.Short;
  const isSwap = tradeType === TradeType.Swap;
  const isPosition = isLong || isShort;
  const isMarket = tradeMode === TradeMode.Market;
  const isLimit = tradeMode === TradeMode.Limit;
  const isTrigger = tradeMode === TradeMode.Trigger;
  const isIncrease = isPosition && (isMarket || isLimit);

  const tradeFlags: TradeFlags = {
    isLong,
    isShort,
    isSwap,
    isPosition,
    isIncrease,
    isMarket,
    isLimit,
    isTrigger,
  };

  return tradeFlags;
};

export const getTradeParamsFromPosition = (position: PositionInfo) => {
  return {
    tradeType: position.isLong ? TradeType.Long : TradeType.Short,
    marketAddress: position.marketTokenAddress.toBase58(),
    collateralAddress: position.collateralTokenAddress.toBase58(),
    toTokenAddress: position.marketInfo.indexTokenAddress.toBase58(),
  } satisfies TradeParams;
};
