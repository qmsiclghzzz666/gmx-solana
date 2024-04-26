/// <reference types="vite/client"/>

interface GMSOLDeployment {
  store: string;
  oracle: string;
  market_tokens: string[];
  tokens: Tokens;
}

declare const __GMSOL_DEPLOYMENT__: GMSOLDeployment | null;
