import { PropsWithChildren, useCallback } from "react";
import Modal from "../Modal/Modal";
import { t } from "@lingui/macro";
import Button from "../Button/Button";
import LoadingDots from "../Common/LoadingDots/LoadingDots";
import { useNeedToInitializeTokenAccounts } from "@/contexts/shared";
import { Address } from "@coral-xyz/anchor";

interface Props {
  isVisible: boolean,
  tokens: Address[],
  onClose: () => void,
}

export function InitializeTokenAccountGuard({
  tokens,
  isVisible,
  children,
  onClose,
}: PropsWithChildren<Props>) {
  const { isSending, needToInitialize, initialize } = useNeedToInitializeTokenAccounts(tokens);
  const isPassed = needToInitialize.length === 0;
  const handleInitializeBoxClose = useCallback(() => {
    if (!isPassed) {
      onClose();
    }
  }, [isPassed, onClose]);

  return (
    <>
      {isVisible && !isPassed && <InitializeTokenAccountBox
        onClose={handleInitializeBoxClose}
        isSending={isSending}
        initialize={initialize}
      />}
      {isPassed && children}
    </>
  );
}

function InitializeTokenAccountBox({
  onClose,
  initialize,
  isSending
}: {
  onClose: () => void,
  initialize: () => Promise<string | undefined>,
  isSending: boolean,
}) {
  const handleClick = useCallback(() => {
    void initialize();
  }, [initialize]);
  return (
    <Modal isVisible onClose={onClose} label={t`Initialize Token Accounts`}>
      <Button
        className="w-full"
        variant="primary-action"
        onClick={handleClick}
        disabled={isSending}
      >
        {isSending ? <LoadingDots size={14} /> : t`Initialize`}
      </Button>
    </Modal>
  );
}
