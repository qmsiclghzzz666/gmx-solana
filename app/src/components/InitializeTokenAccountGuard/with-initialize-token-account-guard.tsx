import { ComponentType } from "react";
import { InitializeTokenAccountGuard } from "./InitializeTokenAccountGuard";
import { Address } from "@coral-xyz/anchor";

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
    const componentProps = props as P;
    return (
      <InitializeTokenAccountGuard isVisible={isVisible} onClose={onClose} tokens={tokens}>
        <Component {...componentProps} />
      </InitializeTokenAccountGuard>
    );
  };
  return Guarded;
}
