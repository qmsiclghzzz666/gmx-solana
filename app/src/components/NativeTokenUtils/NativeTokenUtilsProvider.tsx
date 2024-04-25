import { ReactNode, useCallback, useState } from "react";
import { NativeTokenUtilsContext } from "./context";
import { WrapNativeTokenModal } from "./WrapNativeTokenModal";
import { useDeployedMarketInfos } from "@/onchain";
import { getTokenData } from "@/onchain/token/utils";
import { NATIVE_TOKEN_ADDRESS } from "@/config/tokens";

export const NativeTokenUtilsProvider = ({ children }: { children: ReactNode }) => {
  const [isWrapping, setIsWrapping] = useState(false);
  const { tokens } = useDeployedMarketInfos();
  const nativeToken = getTokenData(tokens, NATIVE_TOKEN_ADDRESS);

  const isNativeTokenReady = nativeToken ? true : false;

  const handleOpenWrapNativeTokenModal = useCallback(() => {
    if (!isNativeTokenReady) {
      throw Error("Native token data not ready");
    }
    setIsWrapping(true);
  }, [isNativeTokenReady]);

  return (
    <NativeTokenUtilsContext.Provider value={{
      isNativeTokenReady,
      isWrapping,
      openWrapNativeTokenModal: handleOpenWrapNativeTokenModal,
    }}>
      {children}
      {nativeToken && <WrapNativeTokenModal
        isVisible={isWrapping}
        nativeToken={nativeToken}
        onSubmitted={() => setIsWrapping(false)}
        onClose={() => setIsWrapping(false)}
      />}
    </NativeTokenUtilsContext.Provider>
  );
};
