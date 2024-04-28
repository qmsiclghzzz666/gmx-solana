import { Token } from "@/onchain/token";
import { PublicKey } from "@solana/web3.js";
import { GMSOLDeployment, TokenConfig, Tokens } from "gmsol";
import { WRAPPED_NATIVE_TOKEN_ADDRESS } from "./tokens";

export interface ParsedGMSOLDeployment {
  store: PublicKey,
  oracle: PublicKey,
  marketTokens: PublicKey[],
  tokens: {
    [address: string]: Token,
  }
}

const parseToken = (address: string, token: TokenConfig) => {
  const tokenAddress = new PublicKey(address);
  return {
    symbol: token.symbol,
    address: tokenAddress,
    decimals: token.decimals,
    feedAddress: new PublicKey(token.feedAddress),
    isWrappedNative: tokenAddress.equals(WRAPPED_NATIVE_TOKEN_ADDRESS),
    isStable: token.isStable,
    priceDecimals: token.priceDecimals,
  } as Token;
};

const parseTokens = (tokens: Tokens) => {
  const ans: { [address: string]: Token } = {};
  for (const address in tokens) {
    ans[address] = parseToken(address, tokens[address]);
  }
  return ans;
};

const parseDeployment = (deployment: GMSOLDeployment | null) => {
  if (deployment) {
    const parsed: ParsedGMSOLDeployment = {
      store: new PublicKey(deployment.store),
      oracle: new PublicKey(deployment.oracle),
      marketTokens: deployment.market_tokens.map(token => new PublicKey(token)),
      tokens: parseTokens(deployment.tokens),
    };
    console.debug("parsed deployment:", parsed);
    return parsed;
  }
}

export const GMSOL_DEPLOYMENT = parseDeployment(__GMSOL_DEPLOYMENT__);
