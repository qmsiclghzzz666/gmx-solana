export interface GMSOLDeployment {
    store: string,
    oracle: string,
    market_tokens: string[],
    tokens: Tokens,
}

export interface Tokens {
    [address: string]: TokenConfig,
}

export interface TokenConfig {
    symbol: string,
    decimals: number,
    feedAddress: string,
    isStable?: boolean,
    priceDecimals?: number,
    wrappedAddress?: string,
}
