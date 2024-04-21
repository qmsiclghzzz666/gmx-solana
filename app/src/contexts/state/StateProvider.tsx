import { useDeployedMarketInfos } from "@/onchain";
import { MarketInfos } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { ReactNode, useMemo } from "react";
import { createContext } from "use-context-selector";

export interface State {
  marketInfos: MarketInfos,
  tokens: Tokens,
  marketTokens: Tokens,
}

export const StateCtx = createContext<State | null>(null);

export function StateProvider({ children }: { children: ReactNode }) {
  const { marketInfos, tokens, marketTokens } = useDeployedMarketInfos();

  const state = useMemo(() => {
    const state: State = {
      marketInfos: marketInfos,
      tokens,
      marketTokens,
    };
    return state;
  }, [marketInfos, tokens, marketTokens]);
  return (
    <StateCtx.Provider value={state}>
      {children}
    </StateCtx.Provider>
  );
}
