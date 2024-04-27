import { MarketState, SharedStates, SharedStatesCtx } from ".."
import { useContextSelector, useContext, Context } from "use-context-selector";

export const useSharedStatesSelector = <T>(selector: (s: SharedStates) => T) => {
  const state = useContext(SharedStatesCtx);
  if (!state) {
    throw new Error("Used outside of `SharedStatesProvider`");
  }
  return useContextSelector(SharedStatesCtx as Context<SharedStates>, selector);
};

export const useMarketStateSelector = <T>(selector: (s: MarketState) => T) => {
  return useSharedStatesSelector(s => selector(s.market));
};
