import { MarketInfo } from "@/onchain/market";
import { TokenData } from "@/onchain/token";
import { Dispatch, ReactNode, createContext } from "react";
import { useImmerReducer } from "use-immer";

export const GmStateContext = createContext<GmState | null>(null);
export const GmStateUpdaterContext = createContext<Dispatch<Action> | null>(null);

interface Ctx {
  isDeposit: boolean,
}

export default function GmStateProvider({
  children,
  market,
  firstToken,
  secondToken,
}: Ctx & {
  children: ReactNode,
  market: MarketInfo,
  firstToken?: TokenData,
  secondToken?: TokenData,
}) {
  const [state, dispath] = useImmerReducer(stateReducer, {
    market,
    firstToken,
    secondToken,
  });

  return (
    <GmStateContext.Provider value={state}>
      <GmStateUpdaterContext.Provider value={dispath}>
        {children}
      </GmStateUpdaterContext.Provider>
    </GmStateContext.Provider>
  );
}

interface GmState {
  market: MarketInfo,
  firstToken?: TokenData,
  secondToken?: TokenData,
}

interface Action {
  type: "update-first-token-amount",
}

const stateReducer = (state: GmState, action: Action) => {
  switch (action.type) {
    case 'update-first-token-amount': {
      break;
    }
  }
};
