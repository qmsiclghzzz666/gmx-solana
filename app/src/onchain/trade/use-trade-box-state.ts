import { Dispatch, SetStateAction, useCallback, useEffect, useMemo, useState } from "react";
import { AvailableTokenOptions, TradeFlags, TradeMode, TradeOptions, TradeParams, TradeType } from "./types";
import { useLocalStorageSerializeKey } from "@/utils/localStorage";
import { getSyntheticsTradeOptionsKey } from "@/config/localStorage";
import { MarketInfos } from "../market";
import { Tokens } from "../token";
import { useAvailableTokenOptions } from "./use-available-token-options";
import { mapValues, pick } from "lodash";
import { getByKey } from "@/utils/objects";
import { createTradeFlags } from "./utils";
import { useSafeState } from "@/utils/state";

const INITIAL_TRADE_OPTIONS: TradeOptions = {
  tradeType: TradeType.Long,
  tradeMode: TradeMode.Market,
  tokens: {},
  markets: {},
  collateralAddress: undefined,
};

const useTradeOptions = (chainId: string | undefined, availableTokensOptions: AvailableTokenOptions, marketInfos?: MarketInfos) => {
  const [storedOptions, setStoredOptions] = useLocalStorageSerializeKey(
    getSyntheticsTradeOptionsKey(chainId ?? ""),
    INITIAL_TRADE_OPTIONS,
  );

  const [syncedChainId, setSyncedChainId] = useState<string | undefined>(undefined);

  //#region Manual useMemo zone begin
  // Prevent reinitialization when only the order of the tokens changes.
  /* eslint-disable react-hooks/exhaustive-deps */
  const unstableRefAvailableSwapTokensAddresses = availableTokensOptions.swapTokens.map((t) => t.address.toBase58());
  const swapKey = unstableRefAvailableSwapTokensAddresses.sort().join(",");

  const availableSwapTokenAddresses = useMemo(() => {
    return unstableRefAvailableSwapTokensAddresses;
  }, [swapKey]);

  const unstableRefAvailableIndexTokensAddresses = availableTokensOptions.indexTokens.map((t) => t.address.toBase58());
  const indexKey = unstableRefAvailableIndexTokensAddresses.sort().join(",");

  const availableIndexTokenAddresses = useMemo(() => {
    return unstableRefAvailableIndexTokensAddresses;
  }, [indexKey]);

  const unstableRefStrippedMarketInfos = mapValues(marketInfos || {}, (info) =>
    pick(info, ["longTokenAddress", "shortTokenAddress"])
  );

  const strippedMarketInfos = useMemo(() => {
    return unstableRefStrippedMarketInfos;
  }, [JSON.stringify(unstableRefStrippedMarketInfos)]);
  /* eslint-enable react-hooks/exhaustive-deps */
  //#endregion Manual useMemo zone end

  // Handle chain change.
  useEffect(() => {
    if (syncedChainId === chainId) {
      return;
    }

    if (availableIndexTokenAddresses.length === 0) {
      return;
    }

    if (storedOptions?.tokens.indexTokenAddress && availableIndexTokenAddresses.includes(storedOptions.tokens.indexTokenAddress)) {
      setSyncedChainId(chainId);
      return;
    }

    const market = availableTokensOptions.sortedAllMarkets?.at(0);

    if (!market) {
      return;
    }

    setStoredOptions({
      ...INITIAL_TRADE_OPTIONS,
      markets: {
        [market.marketTokenAddress.toBase58()]: {
          longTokenAddress: market.longTokenAddress.toBase58(),
          shortTokenAddress: market.shortTokenAddress.toBase58(),
        }
      },
      tokens: {
        indexTokenAddress: market.indexTokenAddress.toBase58(),
        fromTokenAddress: market.shortTokenAddress.toBase58(),
      }
    });
    setSyncedChainId(chainId);
  }, [availableIndexTokenAddresses, availableTokensOptions.sortedAllMarkets, chainId, setStoredOptions, storedOptions, syncedChainId]);

  const setTradeOptions = useCallback((action: SetStateAction<TradeOptions | undefined>) => {
    setStoredOptions((oldState) => {
      let newState = typeof action === "function" ? action(oldState)! : action!;

      if (newState && (newState.tradeType === TradeType.Long || newState.tradeType === TradeType.Short)) {
        newState = fallbackPositionTokens(newState);
      }
      return newState;
    });

    function fallbackPositionTokens(newState: TradeOptions) {
      const needFromUpdate = !availableSwapTokenAddresses.find((t) => t === newState.tokens.fromTokenAddress);
      const nextFromTokenAddress =
        needFromUpdate && availableSwapTokenAddresses.length
          ? availableSwapTokenAddresses[0]
          : newState.tokens.fromTokenAddress;

      if (nextFromTokenAddress && nextFromTokenAddress !== newState.tokens.fromTokenAddress) {
        newState = {
          ...newState,
          tokens: {
            ...newState.tokens,
            fromTokenAddress: nextFromTokenAddress,
          },
        };
      }

      const needIndexUpdateByAvailableTokens = !availableIndexTokenAddresses.find(
        (t) => t === newState.tokens.indexTokenAddress
      );

      if (needIndexUpdateByAvailableTokens && availableIndexTokenAddresses.length) {
        const updater = setToTokenAddressUpdaterBuilder(
          newState.tradeType,
          availableIndexTokenAddresses[0],
          undefined
        );

        newState = updater(newState);
      }

      const toTokenAddress =
        newState.tradeType === TradeType.Swap
          ? newState.tokens.swapToTokenAddress
          : newState.tokens.indexTokenAddress;
      const marketAddress = toTokenAddress
        ? newState.markets[toTokenAddress]?.[newState.tradeType === TradeType.Long ? "longTokenAddress" : "shortTokenAddress"]
        : undefined;
      const marketInfo = getByKey(strippedMarketInfos, marketAddress);

      const currentCollateralIncludedInCurrentMarket =
        marketInfo &&
        (marketInfo.longTokenAddress.toBase58() === newState.collateralAddress ||
          marketInfo.shortTokenAddress.toBase58() === newState.collateralAddress);

      const needCollateralUpdate = !newState.collateralAddress || !currentCollateralIncludedInCurrentMarket;

      if (needCollateralUpdate && marketInfo) {
        newState = {
          ...newState,
          collateralAddress: marketInfo.shortTokenAddress.toBase58(),
        };
      }

      return newState;
    }
  }, [availableIndexTokenAddresses, availableSwapTokenAddresses, setStoredOptions, strippedMarketInfos]);

  return [storedOptions, setTradeOptions] as [TradeOptions, Dispatch<SetStateAction<TradeOptions>>];
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
  const [fromTokenInputValue, setFromTokenInputValue] = useSafeState("");
  const [toTokenInputValue, setToTokenInputValue] = useSafeState("");
  const [triggerRatioInputValue, setTriggerRatioInputValue] = useSafeState("");
  const [focusedInput, setFocusedInput] = useState<"from" | "to">();

  const { swapTokens } = avaiableTokensOptions;
  const tradeType = tradeOptions.tradeType;
  const tradeMode = tradeOptions.tradeMode;
  const availalbleTradeModes = useMemo(() => {
    if (!tradeType) {
      return [];
    }
    return {
      [TradeType.Long]: [TradeMode.Market, TradeMode.Limit, TradeMode.Trigger],
      [TradeType.Short]: [TradeMode.Market, TradeMode.Limit, TradeMode.Trigger],
      [TradeType.Swap]: [TradeMode.Market, TradeMode.Limit],
    }[tradeType];
  }, [tradeType]);
  const tradeFlags = useMemo(() => createTradeFlags(tradeType, tradeMode), [tradeType, tradeMode]);
  const { isSwap } = tradeFlags;
  const { fromTokenAddress, toTokenAddress, collateralAddress, marketAddress } = getAddresses(tradeFlags, tradeOptions);

  const setTradeType = useCallback((tradeType: TradeType) => {
    setTradeOptions((state) => {
      return {
        ...state,
        tradeType,
      }
    });
  }, [setTradeOptions]);

  const setTradeMode = useCallback((tradeMode: TradeMode) => {
    setTradeOptions((state) => {
      return {
        ...state,
        tradeMode,
      }
    });
  }, [setTradeOptions]);

  const setTradeParams = useCallback((params: TradeParams) => {
    setTradeOptions((state) => {
      const { tradeType, tradeMode, fromTokenAddress, toTokenAddress, marketAddress, collateralAddress } = params;
      const newState = { ...state };

      if (tradeType) {
        newState.tradeType = tradeType;
      }

      if (tradeMode) {
        newState.tradeMode = tradeMode;
      }

      if (fromTokenAddress) {
        newState.tokens.fromTokenAddress = fromTokenAddress;
      }

      if (toTokenAddress) {
        if (tradeType === TradeType.Swap) {
          newState.tokens.swapToTokenAddress = toTokenAddress;
        } else {
          newState.tokens.indexTokenAddress = toTokenAddress;
          if (toTokenAddress && marketAddress) {
            newState.markets[toTokenAddress] = newState.markets[toTokenAddress] || {};
            if (tradeType === TradeType.Long) {
              newState.markets[toTokenAddress].longTokenAddress = marketAddress;
            } else if (tradeType === TradeType.Short) {
              newState.markets[toTokenAddress].shortTokenAddress = marketAddress;
            }
          }
        }
      }

      if (collateralAddress) {
        newState.collateralAddress = collateralAddress;
      }

      return newState;
    });
  }, [setTradeOptions]);

  const setFromTokenAddress = useCallback(
    (tokenAddress?: string) => {
      setTradeOptions((oldState) => {
        return {
          ...oldState,
          tokens: {
            ...oldState.tokens,
            fromTokenAddress: tokenAddress,
          },
        };
      });
    },
    [setTradeOptions]
  );

  const setToTokenAddress = useCallback(
    function setToTokenAddressCallback(tokenAddress: string, marketTokenAddress?: string, tradeType?: TradeType) {
      setTradeOptions(setToTokenAddressUpdaterBuilder(tradeType, tokenAddress, marketTokenAddress));
    },
    [setTradeOptions]
  );

  const switchTokenAddresses = useCallback(() => {
    setTradeOptions((oldState) => {
      const isSwap = oldState?.tradeType === TradeType.Swap;
      const fromTokenAddress = oldState?.tokens.fromTokenAddress;
      const toTokenAddress = isSwap ? oldState?.tokens.swapToTokenAddress : oldState?.tokens.indexTokenAddress;

      if (isSwap) {
        return {
          ...oldState,
          tokens: {
            ...oldState.tokens,
            fromTokenAddress: toTokenAddress,
            swapToTokenAddress: fromTokenAddress,
          },
        };
      }

      return {
        ...oldState,
        tokens: {
          ...oldState.tokens,
          fromTokenAddress: toTokenAddress,
          indexTokenAddress: fromTokenAddress,
        },
      };
    });
  }, [setTradeOptions]);

  // Update Trade Mode.
  useEffect(() => {
    if (!availalbleTradeModes.includes(tradeMode)) {
      setTradeMode(availalbleTradeModes[0]);
    }
  }, [availalbleTradeModes, setTradeMode, tradeMode]);

  // Update Swap Tokens.
  useEffect(
    () => {
      if (!isSwap || !swapTokens.length) {
        return;
      }

      const needFromUpdate = !swapTokens.find((t) => t.address.toBase58() === fromTokenAddress);

      if (needFromUpdate) {
        setFromTokenAddress(swapTokens[0].address.toBase58());
      }

      const needToUpdate = !swapTokens.find((t) => t.address.toBase58() === toTokenAddress);

      if (needToUpdate) {
        setToTokenAddress(swapTokens[0].address.toBase58());
      }
    },
    [fromTokenAddress, isSwap, setFromTokenAddress, setToTokenAddress, swapTokens, toTokenAddress]
  );

  return {
    marketAddress,
    fromTokenAddress,
    setFromTokenAddress,
    toTokenAddress,
    setToTokenAddress,
    collateralAddress,
    avaiableTokensOptions,
    availalbleTradeModes,
    tradeFlags,
    tradeType,
    setTradeType,
    tradeMode,
    setTradeMode,
    fromTokenInputValue,
    setFromTokenInputValue,
    toTokenInputValue,
    setToTokenInputValue,
    triggerRatioInputValue,
    setTriggerRatioInputValue,
    focusedInput,
    setFocusedInput,
    setTradeParams,
    switchTokenAddresses,
  };
}

function setToTokenAddressUpdaterBuilder(
  tradeType: TradeType | undefined,
  tokenAddress: string,
  marketTokenAddress: string | undefined
): (oldState: TradeOptions | undefined) => TradeOptions {
  return function setToTokenAddressUpdater(oldState: TradeOptions | undefined): TradeOptions {
    const isSwap = oldState?.tradeType === TradeType.Swap;
    const newState = JSON.parse(JSON.stringify(oldState)) as TradeOptions;
    if (!newState) {
      return newState;
    }

    if (tradeType) {
      newState.tradeType = tradeType;
    }

    if (isSwap) {
      newState.tokens.swapToTokenAddress = tokenAddress;
    } else {
      newState.tokens.indexTokenAddress = tokenAddress;
      if (tokenAddress && marketTokenAddress) {
        newState.markets[tokenAddress] = newState.markets[tokenAddress] || {};
        if (newState.tradeType === TradeType.Long) {
          newState.markets[tokenAddress].longTokenAddress = marketTokenAddress;
        } else if (newState.tradeType === TradeType.Short) {
          newState.markets[tokenAddress].shortTokenAddress = marketTokenAddress;
        }
      }
    }

    return newState;
  };
}

function getAddresses({ isSwap, isLong }: { isSwap: boolean, isLong: boolean }, tradeOptions: TradeOptions) {
  const fromTokenAddress = tradeOptions.tokens.fromTokenAddress;
  const toTokenAddress = isSwap
    ? tradeOptions.tokens.swapToTokenAddress
    : tradeOptions.tokens.indexTokenAddress;
  const collateralAddress = tradeOptions.collateralAddress;
  const marketAddress = toTokenAddress
    ? tradeOptions?.markets[toTokenAddress]?.[isLong ? "longTokenAddress" : "shortTokenAddress"]
    : undefined;

  return {
    fromTokenAddress,
    toTokenAddress,
    collateralAddress,
    marketAddress,
  }
}
