import { PublicKey } from "@solana/web3.js";
import { GMSOLDeployment } from "gmsol";

export interface ParsedGMSOLDeployment {
  store: PublicKey,
  oracle: PublicKey,
  marketTokens: PublicKey[],
}

const parseDeployment = (deployment?: GMSOLDeployment) => {
  if (deployment) {
    const parsed: ParsedGMSOLDeployment = {
      store: new PublicKey(deployment.store),
      oracle: new PublicKey(deployment.oracle),
      marketTokens: deployment.market_tokens.map(token => new PublicKey(token)),
    };
    return parsed;
  }
}

export const GMSOL_DEPLOYMENT = parseDeployment(window.__GMSOL_DEPLOYMENT__);
