import { Dispatch, SetStateAction, useCallback, useEffect, useState } from "react";
import { AvailableTokenOptions, TradeMode, TradeOptions, TradeType } from "./types";
import { useLocalStorageSerializeKey } from "@/utils/localStorage";
import { getSyntheticsTradeOptionsKey } from "@/config/localStorage";
import { MarketInfos } from "../market";
import { Tokens } from "../token";
import { useAvailableTokenOptions } from "./use-available-token-options";

const INITIAL_TRADE_OPTIONS: TradeOptions = {
  tradeType: TradeType.Long,
  tradeMode: TradeMode.Market,
  tokens: {},
  markets: {},
  collateralAddress: undefined,
};

const useTradeOptions = (chainId: string | undefined, avaiableTokensOptions: AvailableTokenOptions) => {
  const [storedOptions, setStoredOptions] = useLocalStorageSerializeKey(
    getSyntheticsTradeOptionsKey(chainId ?? ""),
    INITIAL_TRADE_OPTIONS,
  );
  const [syncedChainId, setSyncedChainId] = useState<string | undefined>(undefined);

  // Handle chain change.
  useEffect(() => {
    if (syncedChainId === chainId) {
      return;
    }
    console.log("available token options", avaiableTokensOptions);
    console.log(`stored trade options for ${chainId}`, storedOptions);

    setSyncedChainId(chainId);
  }, [avaiableTokensOptions, chainId, storedOptions, syncedChainId]);

  return [storedOptions!, setStoredOptions] as [TradeOptions, Dispatch<React.SetStateAction<TradeOptions>>];
};

export function useTradeBoxState(
  chainId: string | undefined,
  {
    marketInfos,
    tokens
  }: {
    marketInfos?: MarketInfos,
    tokens?: Tokens,
  },
) {

  const avaiableTokensOptions = useAvailableTokenOptions({ marketInfos, tokens });
  const [tradeOptions, setTradeOptions] = useTradeOptions(chainId, avaiableTokensOptions);

  const tradeType = tradeOptions.tradeType;
  const tradeMode = tradeOptions.tradeMode;

  return {
    tradeType,
    tradeMode,
  };
}
