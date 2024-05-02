import { useCallback } from "react";
import Modal from "../Modal/Modal";
import { Trans, t } from "@lingui/macro";
import Button from "../Button/Button";
import LoadingDots from "../Common/LoadingDots/LoadingDots";
import { TokenData } from "@/onchain/token";
import TokenWithIcon from "../TokenIcon/TokenWithIcon";
import { MarketInfo } from "@/onchain/market";
import { getMarketIndexName } from "../MarketsList/utils";

interface Props {
  isSending: boolean,
  isVisible: boolean,
  tokens: TokenData[],
  marketTokens: MarketInfo[],
  initialize: () => Promise<string | undefined>,
  onClose: () => void,
}

export function InitializeTokenAccountBox({
  isVisible,
  isSending,
  tokens,
  marketTokens,
  onClose,
  initialize,
}: Props) {
  const handleClick = useCallback(() => {
    void initialize();
  }, [initialize]);
  return (
    <div className="Confirmation-box">
      <Modal isVisible={isVisible} onClose={onClose} label={t`Need to initialize Token Accounts`}>
        <span className="muted font-sm"><Trans>The following tokens accounts are required:</Trans></span>
        <div className="Confirmation-box-main">
          {tokens.map(token => {
            return (
              <div key={token.address.toBase58()} className="Confirmation-box-row">
                <span>
                  <TokenWithIcon symbol={token.symbol} displaySize={20} />
                </span>
              </div>
            );
          })}
          {marketTokens.map(market => {
            return (
              <div key={market.marketTokenAddress.toBase58()} className="Confirmation-box-row">
                <span>
                  <TokenWithIcon symbol={market.indexToken.symbol} displaySize={20} name={`GM:${getMarketIndexName(market)}`} />
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
            {isSending ? <LoadingDots size={14} /> : t`Initialize All`}
          </Button>
        </div>
      </Modal>
    </div>
  );
}
