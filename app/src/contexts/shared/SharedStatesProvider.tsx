import { useDeployedMarketInfos } from "@/onchain/market";
import { ReactNode, useMemo } from "react";
import { SharedStates } from "./types";
import { SharedStatesCtx } from ".";

export function SharedStatesProvider({ children }: { children: ReactNode }) {
  const { marketInfos, tokens, marketTokens } = useDeployedMarketInfos();

  const state = useMemo(() => {
    const state: SharedStates = {
      market: {
        marketInfos: marketInfos,
        tokens,
        marketTokens,
      },
      tradeBox: {}
    };
    return state;
  }, [marketInfos, tokens, marketTokens]);
  return (
    <SharedStatesCtx.Provider value={state}>
      {children}
    </SharedStatesCtx.Provider>
  );
}
