import { BN } from "@coral-xyz/anchor";
import { TRIGGER_PREFIX_ABOVE, TRIGGER_PREFIX_BELOW } from "@/config/ui";
import { Token } from "@/onchain/token";

export function getMarketIndexName({ indexToken }: { indexToken: Token }) {
  return `${indexToken.symbol}/USD`
}

export function getMarketPoolName({ longToken, shortToken }: { longToken: Token, shortToken: Token }) {
  if (longToken.address == shortToken.address) {
    return longToken.symbol;
  } else {
    return `${longToken.symbol}-${shortToken.symbol}`;
  }
}

const MAX_EXCEEDING_THRESHOLD = "1000000000";
const MIN_EXCEEDING_THRESHOLD_SCALE = 2;
export const USD_DECIMALS = 20;

export function formatUsd(
  usd?: BN,
  opts: {
    fallbackToZero?: boolean;
    displayDecimals?: number;
    maxThreshold?: string;
    minThreshold?: string;
    displayPlus?: boolean;
  } = {}
) {
  const { fallbackToZero = false, displayDecimals = 2 } = opts;

  if (!usd) {
    if (fallbackToZero) {
      usd = new BN(0);
    } else {
      return undefined;
    }
  }

  const exceedingInfo = getLimitedDisplay(usd, USD_DECIMALS, opts);

  const maybePlus = opts.displayPlus ? "+" : "";
  const sign = usd.lt(new BN(0)) ? "-" : maybePlus;
  const symbol = exceedingInfo.symbol ? `${exceedingInfo.symbol} ` : "";
  const displayUsd = formatAmount(exceedingInfo.value, USD_DECIMALS, displayDecimals, true);
  return `${symbol}${sign}$${displayUsd}`;
}

function getLimitedDisplay(
  amount: BN,
  tokenDecimals: number,
  opts: { maxThreshold?: string; minThresholdScale?: number } = {}
) {
  const { maxThreshold = MAX_EXCEEDING_THRESHOLD, minThresholdScale = MIN_EXCEEDING_THRESHOLD_SCALE } = opts;
  const max = expandDecimals(new BN(maxThreshold), tokenDecimals);
  const min = new BN(10).pow(new BN(tokenDecimals - minThresholdScale));
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


export function formatRatePercentage(rate?: BN, displayDecimals?: number) {
  if (!rate) {
    return "-";
  }

  return `${getPlusOrMinusSymbol(rate)}${formatAmount(rate.mul(new BN(100)).abs(), 30, displayDecimals ?? 4)}%`;
}

export function getUnit(decimals: number) {
  return (new BN(10)).pow(new BN(decimals));
}

export function expandDecimals(n: BN, decimals: number) {
  return n.mul(getUnit(decimals));
}

export const formatAmount = (
  amount: BN | null,
  tokenDecimals: number,
  displayDecimals?: number,
  useCommas?: boolean,
  defaultValue?: string
) => {
  if (!defaultValue) {
    defaultValue = "...";
  }
  if (amount === null || amount.toString().length === 0) {
    return defaultValue;
  }
  if (displayDecimals === undefined) {
    displayDecimals = 4;
  }
  let amountStr = toFixedDecimal(amount, tokenDecimals);
  amountStr = limitDecimals(amountStr, displayDecimals);
  if (displayDecimals !== 0) {
    amountStr = padDecimals(amountStr, displayDecimals);
  }
  if (useCommas) {
    return numberWithCommas(amountStr);
  }
  return amountStr;
};

export function toFixedDecimal(amount: BN, decimals: number) {
  const ten = new BN(10);
  const divisor = ten.pow(new BN(decimals));
  const integerPart = amount.div(divisor);
  const decimalPart = amount.mod(divisor).toString(10, decimals);

  return `${integerPart.toString()}.${decimalPart}`;
}

export function getPlusOrMinusSymbol(value?: BN, opts: { showPlusForZero?: boolean } = {}): string {
  if (!value) {
    return "";
  }

  const { showPlusForZero = false } = opts;
  return value.isZero() ? (showPlusForZero ? "+" : "") : value.isNeg() ? "-" : "+";
}


export const limitDecimals = (amountStr: string, maxDecimals?: number) => {
  if (maxDecimals === undefined) {
    return amountStr;
  }
  if (maxDecimals === 0) {
    return amountStr.split(".")[0];
  }
  const dotIndex = amountStr.indexOf(".");
  if (dotIndex !== -1) {
    const decimals = amountStr.length - dotIndex - 1;
    if (decimals > maxDecimals) {
      amountStr = amountStr.substring(0, amountStr.length - (decimals - maxDecimals));
    }
  }

  return amountStr;
};

export const padDecimals = (amountStr: string, minDecimals: number) => {
  const dotIndex = amountStr.indexOf(".");
  if (dotIndex !== -1) {
    const decimals = amountStr.length - dotIndex - 1;
    if (decimals < minDecimals) {
      amountStr = amountStr.padEnd(amountStr.length + (minDecimals - decimals), "0");
    }
  } else {
    amountStr = amountStr + ".0000";
  }
  return amountStr;
};

export function numberWithCommas(amountStr: string) {
  if (!amountStr) {
    return "...";
  }

  const parts = amountStr.split(".");
  parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  return parts.join(".");
}
