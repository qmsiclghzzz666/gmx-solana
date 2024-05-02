import { formatAmount, formatUsd, getPlusOrMinusSymbol, limitDecimals, toFixedDecimal } from "@/components/MarketsList/utils";
import { BN_ZERO, ONE_USD, USD_DECIMALS } from "@/config/constants";
import { BASIS_POINTS_DIVISOR } from "@/config/factors";
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

/**
 * Converts a numeric string to a BigNumber representation based on the specified unit.
 * @param {string} value - The numeric string to convert.
 * @param {number} decimals - The number of decimal places to account for the unit (e.g., 18 for ether to wei conversion).
 * @returns {BN} The BigNumber object representing the value.
 */
export function parseUnits(value: string, decimals = 18) {
  // Ensure the input is a string
  if (typeof value !== 'string') {
    throw new TypeError('Value must be a string');
  }

  const parts = value.split('.');
  const integerPart = parts[0];
  const decimalPart = parts[1] || '';
  if (decimalPart.length > decimals) {
    throw new Error('Decimal places exceed decimals limit');
  }

  // Extend the decimal part to the specified number of decimals
  const fullDecimalPart = (decimalPart + '0'.repeat(decimals)).substring(0, decimals);

  // Combine the integer part with the extended decimal part
  const fullNumber = integerPart + fullDecimalPart;

  // Remove leading zeros
  const cleanNumber = fullNumber.replace(/^0+/, '') || '0';

  return new BN(cleanNumber);
}

export const parseValue = (value: string, tokenDecimals: number) => {
  const pValue = parseFloat(value);

  if (isNaN(pValue)) {
    return undefined;
  }
  value = limitDecimals(value, tokenDecimals);
  return parseUnits(value, tokenDecimals);
};

export function bnClampMin(value: BN, min: BN) {
  return value.lt(min) ? min : value;
}

export function toBigInt(amount: BN) {
  return BigInt(amount.toString());
}

export function getBasisPoints(numerator: BN, denominator: BN, shouldRoundUp = false) {
  const result = numerator.muln(BASIS_POINTS_DIVISOR).div(denominator);

  if (shouldRoundUp) {
    const remainder = numerator.muln(BASIS_POINTS_DIVISOR).mod(denominator);
    if (!remainder.isZero()) {
      return result.isNeg() ? result.subn(1) : result.addn(1);
    }
  }

  return result;
}

export function formatDeltaUsd(
  deltaUsd?: BN,
  percentage?: BN,
  opts: { fallbackToZero?: boolean; showPlusForZero?: boolean } = {}
) {
  if (!deltaUsd) {
    if (opts.fallbackToZero) {
      return `${formatUsd(BN_ZERO)} (${formatAmount(BN_ZERO, 2, 2)}%)`;
    }

    return undefined;
  }

  const sign = getPlusOrMinusSymbol(deltaUsd, { showPlusForZero: opts.showPlusForZero });

  const exceedingInfo = getLimitedDisplay(deltaUsd, USD_DECIMALS);
  const percentageStr = percentage ? ` (${sign}${formatPercentage(percentage.abs())})` : "";
  const deltaUsdStr = formatAmount(exceedingInfo.value, USD_DECIMALS, 2, true);
  const symbol = exceedingInfo.symbol ? `${exceedingInfo.symbol} ` : "";

  return `${symbol}${sign}$${deltaUsdStr}${percentageStr}`;
}

export function formatPercentage(percentage?: BN, opts: { fallbackToZero?: boolean; signed?: boolean } = {}) {
  const { fallbackToZero = false, signed = false } = opts;

  if (!percentage) {
    if (fallbackToZero) {
      return `${formatAmount(BN_ZERO, 2, 2)}%`;
    }

    return undefined;
  }

  const sign = signed ? getPlusOrMinusSymbol(percentage) : "";

  return `${sign}${formatAmount(percentage.abs(), 2, 2)}%`;
}

export function applyFactor(value: BN, factor: BN) {
  return value.mul(factor).div(ONE_USD);
}
