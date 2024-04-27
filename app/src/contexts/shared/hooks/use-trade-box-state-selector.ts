import { TradeBoxState } from "@/onchain/trade";
import { useSharedStatesSelector } from "./use-shared-states-selector";

export const useTradeBoxStateSelector = <T>(selector: (s: TradeBoxState) => T) => {
  return useSharedStatesSelector(s => selector(s.tradeBox));
};
