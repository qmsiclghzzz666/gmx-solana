import { IncreaseSwapParams } from "@/onchain/trade";
import { selectCollateralToken, selectFromToken, selectTradeBoxTradeFlags } from ".";
import { createSharedStatesSelector } from "../../utils";
import { selectMarketGraph } from "../market-selectors";
import dijkstra from 'graphology-shortest-path/dijkstra';
import { edgePathFromNodePath } from 'graphology-shortest-path/utils';
import { edgeNameToMarketTokenAddress } from "@/onchain/market";

export const selectIncreaseSwapParams = createSharedStatesSelector([
  selectTradeBoxTradeFlags,
  selectFromToken,
  selectCollateralToken,
  selectMarketGraph,
], ({ isIncrease }, fromToken, toToken, marketGraph) => {
  if (!isIncrease || !fromToken || !toToken) return;
  let path: string[] = [];
  let swapTokens: string[] = [];
  try {
    swapTokens = dijkstra.bidirectional(marketGraph, fromToken.address.toBase58(), toToken.address.toBase58(), "fee");
    path = edgePathFromNodePath(marketGraph, swapTokens).map(edgeNameToMarketTokenAddress);
  } catch (error) {
    console.error("find swap path error:", error);
  }
  return {
    initialCollateralToken: fromToken,
    swapPath: path,
    swapTokens,
  } as IncreaseSwapParams;
});
