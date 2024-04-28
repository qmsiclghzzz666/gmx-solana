import { ReactNode, useCallback, useState } from "react";
import { NativeTokenUtilsContext } from "./context";
import { WrapNativeTokenModal } from "./WrapNativeTokenModal";
import { useDeployedMarketInfos } from "@/onchain/market";
import { getTokenData } from "@/onchain/token";
import { NATIVE_TOKEN_ADDRESS, WRAPPED_NATIVE_TOKEN_ADDRESS } from "@/config/tokens";
import { UnwrapNativeTokenModal } from "./UnwrapNativeTokenModal";

export const NativeTokenUtilsProvider = ({ children }: { children: ReactNode }) => {
  const [isWrapping, setIsWrapping] = useState(false);
  const [isUnwrapping, setIsUnwrapping] = useState(false);

  const { tokens } = useDeployedMarketInfos();
  const nativeToken = getTokenData(tokens, NATIVE_TOKEN_ADDRESS);
  const wrappedNativeToken = getTokenData(tokens, WRAPPED_NATIVE_TOKEN_ADDRESS);

  const isNativeTokenReady = nativeToken && wrappedNativeToken ? true : false;

  const handleOpenWrapNativeTokenModal = useCallback(() => {
    if (!isNativeTokenReady) {
      throw Error("Native token data not ready");
    }
    setIsWrapping(true);
  }, [isNativeTokenReady]);

  const handleOpenUnwrapNativeTokenModal = useCallback(() => {
    if (!isNativeTokenReady) {
      throw Error("Native token data not ready");
    }
    setIsUnwrapping(true);
  }, [isNativeTokenReady]);

  return (
    <NativeTokenUtilsContext.Provider value={{
      isNativeTokenReady,
      isWrapping,
      isUnwrapping,
      openWrapNativeTokenModal: handleOpenWrapNativeTokenModal,
      openUnwrapNativeTokenModal: handleOpenUnwrapNativeTokenModal,
    }}>
      {children}
      {nativeToken && <WrapNativeTokenModal
        isVisible={isWrapping}
        nativeToken={nativeToken}
        onSubmitted={() => setIsWrapping(false)}
        onClose={() => setIsWrapping(false)}
      />}
      {wrappedNativeToken && <UnwrapNativeTokenModal
        isVisible={isUnwrapping}
        wrappedNativeToken={wrappedNativeToken}
        onSubmitted={() => setIsUnwrapping(false)}
        onClose={() => setIsUnwrapping(false)} />
      }
    </NativeTokenUtilsContext.Provider>
  );
};
