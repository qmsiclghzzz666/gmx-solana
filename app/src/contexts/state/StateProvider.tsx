import { useDeployedMarketInfos } from "@/onchain";
import { ReactNode, useMemo } from "react";
import { State } from "./types";
import { StateCtx } from ".";

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
