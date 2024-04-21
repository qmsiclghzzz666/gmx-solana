import { formatAmount, formatUsd, limitDecimals, toFixedDecimal } from "@/components/MarketsList/utils";
import { BN_ZERO } from "@/config/constants";
import { TRIGGER_PREFIX_ABOVE, TRIGGER_PREFIX_BELOW } from "@/config/ui";
import { BN } from "@coral-xyz/anchor";

const MAX_EXCEEDING_THRESHOLD = "1000000000";
const MIN_EXCEEDING_THRESHOLD_SCALE = 2;

export function getUnit(decimals: number) {
  return (new BN(10)).pow(new BN(decimals));
}

export function expandDecimals(n: BN, decimals: number) {
  return n.mul(getUnit(decimals));
}

export function convertToUsd(
  tokenAmount: BN | undefined,
  tokenDecimals: number | undefined,
  price: BN | undefined
) {
  if (!tokenAmount || typeof tokenDecimals !== "number" || !price) {
    return undefined;
  }

  return tokenAmount.mul(price).div(expandDecimals(new BN(1), tokenDecimals));
}

export const trimZeroDecimals = (amount: string) => {
  if (parseFloat(amount) === parseInt(amount)) {
    return parseInt(amount).toString();
  }
  return amount;
};

export const formatAmountFree = (amount: BN, tokenDecimals: number, displayDecimals?: number) => {
  if (!amount) {
    return "...";
  }
  let amountStr = toFixedDecimal(amount, tokenDecimals);
  amountStr = limitDecimals(amountStr, displayDecimals);
  return trimZeroDecimals(amountStr);
};

export function getLimitedDisplay(
  amount: BN,
  tokenDecimals: number,
  opts: { maxThreshold?: string; minThresholdScale?: number; minThreshold?: number } = {}
) {
  const { maxThreshold = MAX_EXCEEDING_THRESHOLD, minThresholdScale = MIN_EXCEEDING_THRESHOLD_SCALE, minThreshold = 1 } = opts;
  const max = expandDecimals(new BN(maxThreshold), tokenDecimals);
  const min = new BN(minThreshold).mul(new BN(10).pow(new BN(tokenDecimals - minThresholdScale)));
  const absAmount = amount.abs();

  if (absAmount.isZero()) {
    return {
      symbol: "",
      value: absAmount,
    };
  }

  const symbol = absAmount.gt(max) ? TRIGGER_PREFIX_ABOVE : absAmount.lt(min) ? TRIGGER_PREFIX_BELOW : "";
  const value = absAmount.gt(max) ? max : absAmount.lt(min) ? min : absAmount;

  return {
    symbol,
    value,
  };
}

export function formatTokenAmount(
  amount?: BN,
  tokenDecimals?: number,
  symbol?: string,
  opts: {
    showAllSignificant?: boolean;
    displayDecimals?: number;
    fallbackToZero?: boolean;
    useCommas?: boolean;
    minThreshold?: number;
    minThresholdScale?: number;
    maxThreshold?: string;
    displayPlus?: boolean;
  } = {}
) {
  const {
    displayDecimals = 4,
    showAllSignificant = false,
    fallbackToZero = false,
    useCommas = false,
    minThreshold = 0,
    minThresholdScale = 0,
    maxThreshold,
  } = opts;

  const symbolStr = symbol ? ` ${symbol}` : "";

  if (!amount || !tokenDecimals) {
    if (fallbackToZero) {
      amount = BN_ZERO;
      tokenDecimals = displayDecimals;
    } else {
      return undefined;
    }
  }

  let amountStr: string;

  const maybePlus = opts.displayPlus ? "+" : "";
  const sign = amount.lt(BN_ZERO) ? "-" : maybePlus;

  if (showAllSignificant) {
    amountStr = formatAmountFree(amount, tokenDecimals, tokenDecimals);
  } else {
    const exceedingInfo = getLimitedDisplay(amount, tokenDecimals, { maxThreshold, minThreshold, minThresholdScale });
    const symbol = exceedingInfo.symbol ? `${exceedingInfo.symbol} ` : "";
    amountStr = `${symbol}${sign}${formatAmount(exceedingInfo.value, tokenDecimals, displayDecimals, useCommas)}`;
  }

  return `${amountStr}${symbolStr}`;
}

export function formatTokenAmountWithUsd(
  tokenAmount?: BN,
  usdAmount?: BN,
  tokenSymbol?: string,
  tokenDecimals?: number,
  opts: {
    fallbackToZero?: boolean;
    displayDecimals?: number;
    displayPlus?: boolean;
  } = {}
) {
  if (!tokenAmount || !usdAmount || !tokenSymbol || !tokenDecimals) {
    if (!opts.fallbackToZero) {
      return undefined;
    }
  }

  const tokenStr = formatTokenAmount(tokenAmount, tokenDecimals, tokenSymbol, {
    ...opts,
    useCommas: true,
    displayPlus: opts.displayPlus,
  });

  const usdStr = formatUsd(usdAmount, {
    fallbackToZero: opts.fallbackToZero,
    displayPlus: opts.displayPlus,
  });

  return `${tokenStr} (${usdStr})`;
}
