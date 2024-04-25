import { createContext } from "react";

export interface NativeTokenUtils {
  isNativeTokenReady: boolean,
  isWrapping: boolean,
  openWrapNativeTokenModal: () => void,
}

export const NativeTokenUtilsContext = createContext<NativeTokenUtils | null>(null);
