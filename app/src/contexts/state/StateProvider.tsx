import { useDeployedMarketInfos } from "@/onchain/market";
import { ReactNode, useMemo } from "react";
import { State } from "./types";
import { StateCtx } from ".";

export function StateProvider({ children }: { children: ReactNode }) {
  const { marketInfos, tokens, marketTokens } = useDeployedMarketInfos();

  const state = useMemo(() => {
    const state: State = {
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
    <StateCtx.Provider value={state}>
      {children}
    </StateCtx.Provider>
  );
}
