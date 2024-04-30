import { TokenPrices } from "@/onchain/token";

export function getShouldUseMaxPrice(isIncrease: boolean, isLong: boolean) {
  return isIncrease ? isLong : !isLong;
}

export function getMarkPrice({
  prices,
  isIncrease,
  isLong,
}: {
  prices: TokenPrices,
  isIncrease: boolean,
  isLong: boolean,
}) {
  const shouldUseMaxPrice = getShouldUseMaxPrice(isIncrease, isLong);
  return shouldUseMaxPrice ? prices.maxPrice : prices.minPrice;
}
