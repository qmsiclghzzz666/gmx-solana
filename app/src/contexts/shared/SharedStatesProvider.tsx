import { useDeployedMarketInfos } from "@/onchain/market";
import { ReactNode, useMemo } from "react";
import { SharedStates } from "./types";
import { SharedStatesCtx } from ".";
import { useTradeBoxState } from "@/onchain/trade";
import { useGenesisHash } from "@/onchain/utils";

export function SharedStatesProvider({ children }: { children: ReactNode }) {
  const chainId = useGenesisHash();
  const { marketInfos, tokens, marketTokens } = useDeployedMarketInfos();
  const tradeBox = useTradeBoxState(chainId, { marketInfos, tokens });

  const state = useMemo(() => {
    const state: SharedStates = {
      chainId,
      market: {
        marketInfos: marketInfos,
        tokens,
        marketTokens,
      },
      tradeBox,
    };
    return state;
  }, [chainId, marketInfos, tokens, marketTokens, tradeBox]);
  return (
    <SharedStatesCtx.Provider value={state}>
      {children}
    </SharedStatesCtx.Provider>
  );
}
