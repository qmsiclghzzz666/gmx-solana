import { useDeployedMarketInfos } from "@/onchain";
import { MarketInfos } from "@/onchain/market";
import { ReactNode, useMemo } from "react";
import { createContext } from "use-context-selector";

export interface State {
  marketInfos: MarketInfos,
}

export const StateCtx = createContext<State | null>(null);

export function StateProvider({ children }: { children: ReactNode }) {
  const marketInfos = useDeployedMarketInfos();

  const state = useMemo(() => {
    const state: State = {
      marketInfos: marketInfos,
    };
    return state;
  }, [marketInfos]);
  return (
    <StateCtx.Provider value={state}>
      {children}
    </StateCtx.Provider>
  );
}
