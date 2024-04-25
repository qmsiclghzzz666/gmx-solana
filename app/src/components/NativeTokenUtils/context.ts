import { createContext } from "react";

export interface NativeTokenUtils {
  isNativeTokenReady: boolean,
  isWrapping: boolean,
  isUnwrapping: boolean,
  openWrapNativeTokenModal: () => void,
  openUnwrapNativeTokenModal: () => void,
}

export const NativeTokenUtilsContext = createContext<NativeTokenUtils | null>(null);
