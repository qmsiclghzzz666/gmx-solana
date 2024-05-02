import Button from "@/components/Button/Button";
import LoadingDots from "@/components/Common/LoadingDots/LoadingDots";
import Modal from "@/components/Modal/Modal";
import { t } from "@lingui/macro";

interface Props {
  isPending: boolean,
  isVisible: boolean,
  operationText: string,
  onClose: () => void,
}

export function GmConfirmationBox({
  isPending,
  isVisible,
  operationText,
  onClose,
}: Props) {
  return (
    <div className="Confirmation-box GmConfirmationBox">
      <Modal isVisible={isVisible} onClose={onClose} label={t`Confirm ${operationText}`}>
        {isVisible && (
          <div className="Confirmation-box-row">
            <Button
              className="w-full"
              variant="primary-action"
              type="submit"
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
