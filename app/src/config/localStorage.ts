export const SYNTHETICS_MARKET_DEPOSIT_TOKEN_KEY = "synthetics-market-deposit-token";
export const SYNTHETICS_DEPOSIT_INDEX_TOKEN_KEY = "synthetics-deposit-index-token";

export function getSyntheticsDepositIndexTokenKey(endpoint: string) {
  return [endpoint, SYNTHETICS_DEPOSIT_INDEX_TOKEN_KEY];
}
