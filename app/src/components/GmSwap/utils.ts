import { Market, MarketInfo } from "@/onchain/market";
import { Token, TokenData } from "@/onchain/token";

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

export const getTokenOptions = (marketInfo?: MarketInfo) => {
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

export interface TokenOptions {
  tokenOptions: Token[],
  firstToken?: TokenData,
  secondToken?: TokenData,
}
