import { IncreaseSwapParams } from "@/onchain/trade";
import { selectCollateralToken, selectFromToken, selectTradeBoxTradeFlags } from ".";
import { createSharedStatesSelector } from "../../utils";
import { selectMarketGraph } from "../market-selectors";
import { dijkstraWithLimit, edgeNameToMarketTokenAddress } from "@/onchain/market";

export const MAX_LENGTH = 5;

export const selectIncreaseSwapParams = createSharedStatesSelector([
  selectTradeBoxTradeFlags,
  selectFromToken,
  selectCollateralToken,
  selectMarketGraph,
], ({ isIncrease }, fromToken, toToken, marketGraph) => {
  if (!isIncrease || !fromToken || !toToken) return;
  let swapPath: string[] = [];
  let swapTokens: string[] = [];
  try {
    const { nodePath, edgePath } = dijkstraWithLimit(
      marketGraph,
      fromToken.address.toBase58(),
      toToken.address.toBase58(),
      attrs => attrs['fee'] as number,
      MAX_LENGTH,
    );
    swapPath = edgePath.map(edgeNameToMarketTokenAddress);
    swapTokens = nodePath;
  } catch (error) {
    console.error("find swap path error:", error);
  }
  return {
    isSwapfulfilled: fromToken.address === toToken.address || swapPath.length > 0,
    initialCollateralToken: fromToken,
    swapPath,
    swapTokens,
  } satisfies IncreaseSwapParams as IncreaseSwapParams;
});
