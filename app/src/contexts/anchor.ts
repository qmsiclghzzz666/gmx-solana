import { createContext, useContext, useMemo } from "react";
import { AnchorProvider } from "@coral-xyz/anchor";
import { makeDataStoreProgram } from "gmsol";
import { Connection } from "@solana/web3.js";

export interface AnchorContextType {
  connection: Connection | null,
  provider: AnchorProvider | null,
}

export const AnchorContext = createContext<AnchorContextType>({
  connection: null,
  provider: null,
});

export const useAnchorProvider = () => {
  const ctx = useContext(AnchorContext);
  return ctx.provider
};

export const useDataStore = () => {
  const ctx = useContext(AnchorContext);
  const program = useMemo(() => {
    return ctx.provider ? makeDataStoreProgram(ctx.provider) : ctx.connection ? makeDataStoreProgram({
      connection: ctx.connection
    }) : null;
  }, [ctx]);

  return program;
}
