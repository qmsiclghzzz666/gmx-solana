import { useCallback } from "react";
import Modal from "../Modal/Modal";
import { Trans, t } from "@lingui/macro";
import Button from "../Button/Button";
import LoadingDots from "../Common/LoadingDots/LoadingDots";
import { TokenData } from "@/onchain/token";
import TokenWithIcon from "../TokenIcon/TokenWithIcon";

interface Props {
  isSending: boolean,
  isVisible: boolean,
  tokens: TokenData[],
  initialize: () => Promise<string | undefined>,
  onClose: () => void,
}

export function InitializeTokenAccountBox({
  isVisible,
  isSending,
  tokens,
  onClose,
  initialize,
}: Props) {
  const handleClick = useCallback(() => {
    void initialize();
  }, [initialize]);
  return (
    <div className="Confirmation-box">
      <Modal isVisible={isVisible} onClose={onClose} label={t`Need to initialize Token Accounts`}>
        <span className="muted font-sm"><Trans>Initialize the accounts for the following tokens:</Trans></span>
        <div className="Confirmation-box-main">
          {tokens.map(token => {
            return (
              <div key={token.address.toBase58()}>
                <span>
                  <TokenWithIcon symbol={token.symbol} displaySize={20} />
                </span>
              </div>
            );
          })}
        </div>
        <div className="Confirmation-box-row">
          <Button
            className="w-full"
            variant="primary-action"
            onClick={handleClick}
            disabled={isSending}
          >
            {isSending ? <LoadingDots size={14} /> : t`Initialize`}
          </Button>
        </div>
      </Modal>
    </div>
  );
}
