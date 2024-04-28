import { getTokenData } from "@/onchain/token";
import { createSharedStatesSelector } from "../../utils";
import { selectMarketStateTokens } from "../market-selectors";
import { selectTradeBoxFromTokenAddress, selectTradeBoxToTokenAddress, selectTradeBoxTradeFlags } from "../trade-box-selectors";

export const selectChartToken = createSharedStatesSelector([
  selectTradeBoxFromTokenAddress,
  selectTradeBoxToTokenAddress,
  selectTradeBoxTradeFlags,
  selectMarketStateTokens,
], (fromTokenAddress, toTokenAddress, flags, tokens) => {
  if (!fromTokenAddress || !toTokenAddress) {
    return;
  }
  const { isSwap } = flags;

  const fromToken = getTokenData(tokens, fromTokenAddress);
  const toToken = getTokenData(tokens, toTokenAddress);
  const chartToken = isSwap && toToken?.isStable && !fromToken?.isStable ? fromToken : toToken;
  return chartToken;
});
