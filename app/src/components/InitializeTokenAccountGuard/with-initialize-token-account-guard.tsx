import { ComponentType, useCallback } from "react";
import { InitializeTokenAccountBox } from "./InitializeTokenAccountBox";
import { Address } from "@coral-xyz/anchor";
import { useNeedToInitializeTokenAccounts } from "@/contexts/shared";

interface Props {
  isVisible: boolean,
  onClose: () => void,
}

type GuardedProps<P extends Props> = P & {
  tokens: Address[],
};

export function withInitializeTokenAccountGuard<P extends Props>(Component: ComponentType<P>) {
  const Guarded = (props: GuardedProps<P>) => {
    const { tokens, onClose, isVisible } = props;
    const { isSending, needToInitialize, initialize } = useNeedToInitializeTokenAccounts(tokens);
    const isPassed = needToInitialize.length === 0;
    const handleInitializeBoxClose = useCallback(() => {
      if (!isPassed) {
        onClose();
      }
    }, [isPassed, onClose]);
    const componentProps = { ...props, isVisible: isVisible && isPassed } as P;
    return (
      <>
        <InitializeTokenAccountBox
          isVisible={isVisible && !isPassed}
          onClose={handleInitializeBoxClose}
          isSending={isSending}
          initialize={initialize}
          tokens={needToInitialize}
        />
        <Component {...componentProps} />
      </>
    );
  };
  return Guarded;
}
