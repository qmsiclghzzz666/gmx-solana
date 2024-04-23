import { SYNTHETICS_MARKET_DEPOSIT_TOKEN_KEY } from "@/config/localStorage";
import { MarketInfo } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { getTokenData } from "@/onchain/token/utils";
import { useLocalStorageSerializeKey } from "@/utils/localStorage";
import { Address } from "@coral-xyz/anchor";
import React, { useCallback, useEffect } from "react";
import { Mode, Operation, TokenOptions, getTokenOptions } from "./utils";
import { Context, useContext, useContextSelector } from "use-context-selector";
import { GmState, GmStateContext, GmStateDispatchContext } from "./GmStateProvider";
import { convertToUsd, parseValue } from "@/utils/number";

export const useTokenOptionsFromStorage = ({
  chainId,
  marketInfo,
  operation,
  mode,
  tokensData,
}: {
  chainId: string,
  marketInfo: MarketInfo,
  operation: Operation,
  mode: Mode,
  tokensData?: Tokens,
}) => {
  const isDeposit = operation === Operation.Deposit;
  const isSingle = mode === Mode.Single;
  const isPair = mode === Mode.Pair;
  const tokenOptions = getTokenOptions(marketInfo);

  const [firstTokenAddress, setFirstTokenAddress] = useLocalStorageSerializeKey<string>(
    [chainId, SYNTHETICS_MARKET_DEPOSIT_TOKEN_KEY, isDeposit, marketInfo.marketTokenAddress.toBase58(), "first"],
    ""
  );
  const [secondTokenAddress, setSecondTokenAddress] = useLocalStorageSerializeKey<string>(
    [chainId, SYNTHETICS_MARKET_DEPOSIT_TOKEN_KEY, isDeposit, marketInfo.marketTokenAddress.toBase58(), "second"],
    ""
  );

  // Update tokens.
  useEffect(() => {
    if (!tokenOptions.length) return;

    if (!tokenOptions.find((token) => token.address.toBase58() === firstTokenAddress)) {
      setFirstTokenAddress(tokenOptions[0].address.toBase58());
    }

    if (isSingle && secondTokenAddress) {
      setSecondTokenAddress("");
      return;
    }

    if (isPair && firstTokenAddress) {
      if (marketInfo.isSingle) {
        if (!secondTokenAddress || firstTokenAddress !== secondTokenAddress) {
          setSecondTokenAddress(firstTokenAddress);
        }
        return;
      } else if (firstTokenAddress === secondTokenAddress) {
        setSecondTokenAddress("");
        return;
      }

      if (
        !secondTokenAddress ||
        !tokenOptions.find((token) => token.address.toBase58() === secondTokenAddress) ||
        firstTokenAddress === secondTokenAddress
      ) {
        const secondToken = tokenOptions.find((token) => {
          return (
            token.address.toBase58() !== firstTokenAddress
          );
        });
        setSecondTokenAddress(secondToken?.address.toBase58());
      }
    }
  }, [
    tokenOptions,
    firstTokenAddress,
    setFirstTokenAddress,
    isSingle,
    isPair,
    marketInfo,
    secondTokenAddress,
    setSecondTokenAddress,
  ]);

  const firstToken = getTokenData(tokensData, firstTokenAddress);
  const secondToken = getTokenData(tokensData, secondTokenAddress);
  const updateToken = useCallback((address: Address | null, kind: "first" | "second") => {
    if (kind === "first") {
      setFirstTokenAddress(address?.toString());
    } else if (kind === "second") {
      setSecondTokenAddress(address?.toString());
    }
  }, [setFirstTokenAddress, setSecondTokenAddress]);

  return [
    {
      tokenOptions,
      firstToken,
      secondToken,
    },
    updateToken,
  ] as [TokenOptions, (address: Address | null, kind: "first" | "second") => void];
};

export const useGmStateSelector = <T>(selector: (state: GmState) => T) => {
  if (!useContext(GmStateContext)) {
    throw Error("Cannot use `useGmStateSelector` outside `GmStateProvider`");
  }
  return useContextSelector(GmStateContext as Context<GmState>, selector);
};

export const useGmStateDispath = () => {
  const dispatch = React.useContext(GmStateDispatchContext);
  if (!dispatch) {
    throw Error("Cannot use `useGmStateDispath` outside `GmStateProvider`");
  }
  return dispatch;
};

export const useGmInputDisplay = () => {
  const input = useGmStateSelector(s => s.input);
  const operation = useGmStateSelector(s => s.operation);
  const firstToken = useGmStateSelector(s => s.firstToken);
  const secondToken = useGmStateSelector(s => s.secondToken);
  const marketToken = useGmStateSelector(s => s.marketToken);

  const isDeposit = operation === Operation.Deposit;

  const firstTokenAmount = parseValue(input.firstTokenInputValue, firstToken?.decimals || 0);
  const firstTokenUsd = convertToUsd(
    firstTokenAmount,
    firstToken?.decimals,
    isDeposit ? firstToken?.prices?.minPrice : firstToken?.prices?.maxPrice
  );

  const secondTokenAmount = parseValue(input.secondTokenInputValue, secondToken?.decimals || 0);
  const secondTokenUsd = convertToUsd(
    secondTokenAmount,
    secondToken?.decimals,
    isDeposit ? secondToken?.prices?.minPrice : secondToken?.prices?.maxPrice
  );

  const marketTokenAmount = parseValue(input.marketTokenInputValue || "0", marketToken?.decimals || 0)!;
  const marketTokenUsd = convertToUsd(
    marketTokenAmount,
    marketToken?.decimals,
    isDeposit ? marketToken?.prices?.maxPrice : marketToken?.prices?.minPrice
  )!;

  return {
    firstTokenUsd,
    secondTokenUsd,
    marketTokenUsd,
  };
};
