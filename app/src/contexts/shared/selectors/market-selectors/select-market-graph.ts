import { makeMarketGraph } from "@/onchain/market";
import { createSharedStatesSelector } from "../../utils";
import { selectMarketStateMarketInfos } from "./select-market-state-market-infos";

export const selectMarketGraph = createSharedStatesSelector([selectMarketStateMarketInfos], (marketInfos) => {
  return makeMarketGraph(marketInfos);
});


