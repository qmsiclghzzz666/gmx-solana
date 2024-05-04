import { SYNTHETICS_MARKET_DEPOSIT_TOKEN_KEY } from "@/config/localStorage";
import { Market, MarketInfo } from "@/onchain/market";
import { Tokens } from "@/onchain/token";
import { getTokenData } from "@/onchain/token/utils";
import { useLocalStorageSerializeKey } from "@/utils/localStorage";
import { Address, BN } from "@coral-xyz/anchor";
import React, { useCallback, useEffect } from "react";
import { TokenOptions, getTokenOptions } from "./utils";
import { CreateDepositParams, CreateWithdrawalParams, GmState, Mode, Operation } from "./types";
import { Context, useContext, useContextSelector } from "use-context-selector";
import { GmStateContext, GmStateDispatchContext } from "./context";
import { BN_ZERO } from "@/config/constants";
import { PublicKey } from "@solana/web3.js";
import { selectInputAmounts, selectInputDisplay } from "./selectors";

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
        setSecondTokenAddress(secondToken?.address.toBase58() ?? "");
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
      setFirstTokenAddress(address?.toString() ?? "");
    } else if (kind === "second") {
      setSecondTokenAddress(address?.toString() ?? "");
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

export const useGmInputAmounts = () => useGmStateSelector(selectInputAmounts);

export const useGmInputDisplay = () => useGmStateSelector(selectInputDisplay);

const fixUnnecessarySwap = ({
  market,
  initialLongToken,
  initialShortToken,
  initialLongTokenAmount,
  initialShortTokenAmount,
}: {
  market: Market,
  initialLongToken: PublicKey,
  initialShortToken: PublicKey,
  initialLongTokenAmount?: BN,
  initialShortTokenAmount?: BN
}) => {
  if (initialLongToken.equals(market.shortTokenAddress) && initialShortToken.equals(market.longTokenAddress)) {
    return {
      fixedInitialLongToken: initialShortToken,
      fixedInitialShortToken: initialLongToken,
      fixedInitialLongTokenAmount: initialShortTokenAmount,
      fixedInitialShortTokenAmount: initialLongTokenAmount,
    }
  } else {
    return {
      fixedInitialLongToken: initialLongToken,
      fixedInitialShortToken: initialShortToken,
      fixedInitialLongTokenAmount: initialLongTokenAmount,
      fixedInitialShortTokenAmount: initialShortTokenAmount,
    }
  }
}

export const useHandleSubmit = ({
  onCreateDeposit,
  onCreateWithdrawal,
}: {
  onCreateDeposit: (params: CreateDepositParams) => Promise<void>,
  onCreateWithdrawal: (params: CreateWithdrawalParams) => Promise<void>,
}) => {
  const operation = useGmStateSelector(s => s.operation);
  const mode = useGmStateSelector(s => s.mode);
  const market = useGmStateSelector(s => s.market);
  const initialLongToken = useGmStateSelector(s => s.firstToken?.address) ?? market.longTokenAddress;
  const initialShortToken = useGmStateSelector(s => s.secondToken?.address) ?? market.shortTokenAddress;
  const { firstTokenAmount, secondTokenAmount, marketTokenAmount } = useGmInputAmounts();

  return useCallback(async (skipPreflight: boolean) => {
    if (operation === Operation.Deposit) {
      const initialLongTokenAmount = firstTokenAmount ?? BN_ZERO;
      const initialShortTokenAmount = secondTokenAmount ?? BN_ZERO;

      if (mode === Mode.Single && !initialLongTokenAmount.isZero()) {
        if (initialLongToken.equals(market.shortTokenAddress)) {
          await onCreateDeposit({
            marketToken: market.marketTokenAddress,
            initialLongToken: market.longTokenAddress,
            initialShortToken: initialLongToken,
            initialLongTokenAmount: BN_ZERO,
            initialShortTokenAmount: initialLongTokenAmount,
            skipPreflight,
          });
        } else {
          await onCreateDeposit({
            marketToken: market.marketTokenAddress,
            initialLongToken,
            initialShortToken: market.shortTokenAddress,
            initialLongTokenAmount,
            initialShortTokenAmount: BN_ZERO,
            skipPreflight,
          });
        }
      } else if (mode === Mode.Pair && !(initialLongTokenAmount.isZero() && initialShortTokenAmount.isZero())) {
        const {
          fixedInitialLongToken,
          fixedInitialShortToken,
          fixedInitialLongTokenAmount,
          fixedInitialShortTokenAmount
        } = fixUnnecessarySwap({
          market,
          initialLongToken,
          initialShortToken,
          initialLongTokenAmount,
          initialShortTokenAmount
        });
        await onCreateDeposit({
          marketToken: market.marketTokenAddress,
          initialLongToken: fixedInitialLongToken,
          initialShortToken: fixedInitialShortToken,
          initialLongTokenAmount: fixedInitialLongTokenAmount!,
          initialShortTokenAmount: fixedInitialShortTokenAmount!,
          skipPreflight,
        });
      } else {
        console.log("not enough amounts", mode, initialLongTokenAmount.toString(), initialShortTokenAmount.toString());
      }
    } else if (operation === Operation.Withdrawal && !marketTokenAmount?.isZero()) {
      if (market.isSingle) {
        await onCreateWithdrawal({
          marketToken: market.marketTokenAddress,
          amount: marketTokenAmount,
          finalLongToken: initialLongToken,
          finalShortToken: initialLongToken,
          skipPreflight,
        });
      } else {
        const {
          fixedInitialLongToken,
          fixedInitialShortToken,
        } = fixUnnecessarySwap({
          market,
          initialLongToken,
          initialShortToken,
        });
        await onCreateWithdrawal({
          marketToken: market.marketTokenAddress,
          amount: marketTokenAmount,
          finalLongToken: fixedInitialLongToken,
          finalShortToken: fixedInitialShortToken,
          skipPreflight,
        });
      }
    }
  }, [operation, marketTokenAmount, firstTokenAmount, secondTokenAmount, mode, onCreateDeposit, market, initialLongToken, initialShortToken, onCreateWithdrawal]);
};
