import { GMSOL_DEPLOYMENT } from "@/config/deployment";
import { useMarkets } from "@/onchain/market";

export const useDeployedMarkets = () => {
  return useMarkets(GMSOL_DEPLOYMENT);
};
