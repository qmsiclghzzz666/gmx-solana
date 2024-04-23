import { SYNTHETICS_MARKET_DEPOSIT_TOKEN_KEY } from "@/config/localStorage";
import { Market, MarketInfo } from "@/onchain/market";
import { Token, TokenData, Tokens } from "@/onchain/token";
import { getTokenData } from "@/onchain/token/utils";
import { useLocalStorageSerializeKey } from "@/utils/localStorage";
import { Address } from "@coral-xyz/anchor";
import { useCallback, useEffect } from "react";

export enum Operation {
  Deposit = "Deposit",
  Withdrawal = "Withdrawal",
}

export const parseOperation = (value: string | null) => {
  return value?.toLocaleLowerCase() === "withdrawal" ? Operation.Withdrawal : Operation.Deposit;
}

export enum Mode {
  Single = "Single",
  Pair = "Pair",
}

export const parseMode = (value: string | null) => {
  return value?.toLocaleLowerCase() === "pair" ? Mode.Pair : Mode.Single;
}

export const getGmSwapBoxAvailableModes = (
  operation: Operation,
  market: Pick<Market, "isSingle"> | undefined
) => {
  if (market && market.isSingle) {
    return [Mode.Single];
  }

  if (operation === Operation.Deposit) {
    return [Mode.Single, Mode.Pair];
  }

  return [Mode.Pair];
};

const getTokenOptions = (marketInfo?: MarketInfo) => {
  if (!marketInfo) {
    return [];
  }

  const { longToken, shortToken } = marketInfo;

  if (!longToken || !shortToken) return [];

  const options = [longToken];

  if (!marketInfo.isSingle) {
    options.push(shortToken);
  }

  return options;
};

interface TokenOptions {
  tokenOptions: Token[],
  firstToken?: TokenData,
  secondToken?: TokenData,
}

export const useTokenOptions = ({
  genesisHash,
  marketInfo,
  operation,
  mode,
  tokensData,
}: {
  genesisHash: string,
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
    [genesisHash, SYNTHETICS_MARKET_DEPOSIT_TOKEN_KEY, isDeposit, marketInfo.marketTokenAddress.toBase58(), "first"],
    ""
  );
  const [secondTokenAddress, setSecondTokenAddress] = useLocalStorageSerializeKey<string>(
    [genesisHash, SYNTHETICS_MARKET_DEPOSIT_TOKEN_KEY, isDeposit, marketInfo.marketTokenAddress.toBase58(), "second"],
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
