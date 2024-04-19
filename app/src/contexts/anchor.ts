import { createContext, useContext, useMemo } from "react";
import { AnchorProvider } from "@coral-xyz/anchor";
import { makeDataStoreProgram } from "gmsol";

export interface AnchorContextType {
  provider: AnchorProvider | null,
}

export const AnchorContext = createContext<AnchorContextType>({
  provider: null,
});

export const useAnchorProvider = () => {
  const ctx = useContext(AnchorContext);

  return ctx.provider
};

export const useDataStore = () => {
  const provider = useAnchorProvider();
  const program = useMemo(() => {
    return provider ? makeDataStoreProgram(provider) : null;
  }, [provider]);

  return program;
}
