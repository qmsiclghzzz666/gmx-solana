import { useContext, useMemo } from "react";
import { makeDataStoreProgram, makeExchangeProgram } from "gmsol";
import { AnchorContextCtx } from "./AnchorContextProvider";

export const useDataStore = () => {
  const ctx = useContext(AnchorContextCtx);
  if (!ctx) {
    throw new Error("Used `useDataStore` outside of `AnchorContextProvider`");
  }
  const program = useMemo(() => {
    return ctx.provider ? makeDataStoreProgram(ctx.provider) : makeDataStoreProgram({
      connection: ctx.connection
    });
  }, [ctx.provider, ctx.connection]);

  return program;
}

export const useExchange = () => {
  const ctx = useContext(AnchorContextCtx);
  if (!ctx) {
    throw new Error("Used `useDataStore` outside of `AnchorContextProvider`");
  }
  const program = useMemo(() => {
    return ctx.provider ? makeExchangeProgram(ctx.provider) : makeExchangeProgram({
      connection: ctx.connection
    });
  }, [ctx.provider, ctx.connection]);

  return program;
}
