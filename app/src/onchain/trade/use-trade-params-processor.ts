import { useSharedStatesSelector } from "@/contexts/shared";
import { selectAvailableMarkets, selectMarketAddress, selectSetMarketAddress } from "@/contexts/shared/selectors/trade-box-selectors";
import { useEffect } from "react";

export const useTradeParamsProcessor = () => {
  const marketAddress = useSharedStatesSelector(selectMarketAddress);
  const setMarketAddress = useSharedStatesSelector(selectSetMarketAddress);
  const availablePools = useSharedStatesSelector(selectAvailableMarkets);

  useEffect(() => {
    if (!marketAddress && availablePools.length > 0) {
      setMarketAddress(availablePools[0].marketTokenAddress.toBase58());
    }
  }, [availablePools, marketAddress, setMarketAddress]);
};
