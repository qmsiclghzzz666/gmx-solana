export * from "./NativeTokenUtilsProvider";

import { useContext } from "react";
import { NativeTokenUtilsContext } from "./context";

export const useNativeTokenUtils = () => {
  const ctx = useContext(NativeTokenUtilsContext);
  if (!ctx) {
    throw Error("called outside `NativeTokenUtilsProvider`");
  }
  return ctx;
};
