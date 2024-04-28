export const SYNTHETICS_MARKET_DEPOSIT_TOKEN_KEY = "synthetics-market-deposit-token";
export const SYNTHETICS_DEPOSIT_INDEX_TOKEN_KEY = "synthetics-deposit-index-token";
export const SYNTHETICS_TRADE_OPTIONS = "synthetics-trade-options";

export const LANGUAGE_LOCALSTORAGE_KEY = "LANGUAGE_KEY";

export function getSyntheticsDepositIndexTokenKey(chainId: string) {
  return [chainId, SYNTHETICS_DEPOSIT_INDEX_TOKEN_KEY];
}

export function getSyntheticsTradeOptionsKey(chainId: string) {
  return [chainId, SYNTHETICS_TRADE_OPTIONS];
}
