import { createContext, useContext } from "react";
import { AnchorProvider } from "@coral-xyz/anchor";

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
