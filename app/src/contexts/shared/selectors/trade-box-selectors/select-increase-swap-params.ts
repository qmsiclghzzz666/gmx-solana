import { IncreaseSwapParams } from "@/onchain/trade";
import { selectCollateralToken, selectFromToken, selectTradeBoxTradeFlags } from ".";
import { createSharedStatesSelector } from "../../utils";

export const selectIncreaseSwapParams = createSharedStatesSelector([
  selectTradeBoxTradeFlags,
  selectFromToken,
  selectCollateralToken,
], ({ isIncrease }, fromToken) => {
  if (!isIncrease || !fromToken) return;
  return {
    initialCollateralToken: fromToken,
    swapPath: [],
  } satisfies IncreaseSwapParams;
});
