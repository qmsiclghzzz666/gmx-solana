import { getByKey } from "@/utils/objects";
import { createSharedStatesSelector } from "../../utils";
import { selectMarketStateTokens } from "../market-selectors";
import { selectTradeBoxGetMaxLongShortLiquidityPool } from "./select-trade-box-get-max-long-short-liquidity-pool";
import { selectTradeBoxSetTradeParams } from "./select-trade-box-set-trade-params";
import { selectTradeBoxTradeType } from "./select-trade-box-trade-type";
import { PreferredTradeTypePickStrategy, chooseSuitableMarket } from "@/onchain/market";
import { TradeType } from "@/onchain/trade";

export const selectTradeBoxChooseSuitableMarket = createSharedStatesSelector([
  selectTradeBoxTradeType,
  selectMarketStateTokens,
  selectTradeBoxGetMaxLongShortLiquidityPool,
  selectTradeBoxSetTradeParams,
], (tradeType, tokens, getMaxLongShortLiquidityPool, setTradeParams) => {
  return (tokenAddress: string, preferredTradeType?: PreferredTradeTypePickStrategy) => {
    const token = getByKey(tokens, tokenAddress);

    if (!token) return;

    const { maxLongLiquidityPool, maxShortLiquidityPool } = getMaxLongShortLiquidityPool(token);

    const suitableParams = chooseSuitableMarket({
      indexTokenAddress: tokenAddress,
      maxLongLiquidityPool,
      maxShortLiquidityPool,
      isSwap: tradeType === TradeType.Swap,
      // positionsInfo,
      preferredTradeType: preferredTradeType ?? tradeType,
    });

    if (!suitableParams) return;

    setTradeParams({
      collateralAddress: suitableParams.collateralTokenAddress,
      toTokenAddress: suitableParams.indexTokenAddress,
      marketAddress: suitableParams.marketTokenAddress,
      tradeType: suitableParams.tradeType,
    });

    return suitableParams;
  };
});
