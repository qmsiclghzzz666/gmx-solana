import { Token } from "@/onchain/token";
import { PublicKey } from "@solana/web3.js";
import { GMSOLDeployment, TokenConfig, Tokens } from "gmsol";

export interface ParsedGMSOLDeployment {
  store: PublicKey,
  oracle: PublicKey,
  marketTokens: PublicKey[],
  tokens: {
    [address: string]: Token,
  }
}

const parseToken = (address: string, token: TokenConfig) => {
  return {
    symbol: token.symbol,
    address: new PublicKey(address),
    decimals: token.decimals,
    feedAddress: new PublicKey(token.feedAddress),
  } as Token;
};

const parseTokens = (tokens: Tokens) => {
  const ans: { [address: string]: Token } = {};
  for (const address in tokens) {
    ans[address] = parseToken(address, tokens[address]);
  }
  return ans;
};

const parseDeployment = (deployment?: GMSOLDeployment) => {
  if (deployment) {
    const parsed: ParsedGMSOLDeployment = {
      store: new PublicKey(deployment.store),
      oracle: new PublicKey(deployment.oracle),
      marketTokens: deployment.market_tokens.map(token => new PublicKey(token)),
      tokens: parseTokens(deployment.tokens),
    };
    return parsed;
  }
}

export const GMSOL_DEPLOYMENT = parseDeployment(window.__GMSOL_DEPLOYMENT__);
