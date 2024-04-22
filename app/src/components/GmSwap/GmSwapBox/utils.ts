import { Market } from "@/onchain/market";

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
