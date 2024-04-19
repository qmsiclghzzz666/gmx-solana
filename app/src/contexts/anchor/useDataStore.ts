import { useContext, useMemo } from "react";
import { makeDataStoreProgram } from "gmsol";
import { AnchorContext } from "./AnchorContextProvider";

export const useDataStore = () => {
  const ctx = useContext(AnchorContext);
  const program = useMemo(() => {
    return ctx.provider ? makeDataStoreProgram(ctx.provider) : ctx.connection ? makeDataStoreProgram({
      connection: ctx.connection
    }) : null;
  }, [ctx.provider, ctx.connection]);

  return program;
}
