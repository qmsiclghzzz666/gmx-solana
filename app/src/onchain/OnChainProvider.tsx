import { ReactNode } from "react";
import { SWRConfig, SWRConfiguration } from "swr";
import { AnchorStateProvider } from "@/contexts/anchor";

export function OnChainProvider({ children, refreshInterval = 5000 }: { children: ReactNode, refreshInterval?: number }) {
  return (
    <SWRConfig value={createOnChainSWRConfig(refreshInterval)}>
      <AnchorStateProvider>
        {children}
      </AnchorStateProvider>
    </SWRConfig>
  );
}

const createOnChainSWRConfig = (refreshInterval?: number) => {
  return {
    refreshInterval,
  } as SWRConfiguration;
};
