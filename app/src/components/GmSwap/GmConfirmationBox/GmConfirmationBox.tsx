import Button from "@/components/Button/Button";
import LoadingDots from "@/components/Common/LoadingDots/LoadingDots";
import { withInitializeTokenAccountGuard } from "@/components/InitializeTokenAccountGuard";
import Modal from "@/components/Modal/Modal";
import { t } from "@lingui/macro";
import { useCallback } from "react";

interface Props {
  isPending: boolean,
  isVisible: boolean,
  operationText: string,
  onClose: () => void,
  onSubmit: () => Promise<void>,
  onSubmitted?: () => void,
}

export const GmConfirmationBox = withInitializeTokenAccountGuard(GmConfirmationBoxInner);

function GmConfirmationBoxInner({
  isPending,
  isVisible,
  operationText,
  onClose,
  onSubmit,
  onSubmitted,
}: Props) {
  const handleSubmit = useCallback(() => {
    void onSubmit().then(onSubmitted);
  }, [onSubmit, onSubmitted]);
  return (
    <div className="Confirmation-box GmConfirmationBox">
      <Modal isVisible={isVisible} onClose={onClose} label={t`Confirm ${operationText}`}>
        {isVisible && (
          <div className="Confirmation-box-row">
            <Button
              className="w-full"
              variant="primary-action"
              onClick={handleSubmit}
              disabled={isPending}
            >
              {isPending ? <LoadingDots /> : t`Confirm`}
            </Button>
          </div>
        )}
      </Modal>
    </div>
  );
}
