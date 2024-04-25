import { useCallback, useState } from "react";
import "./ConnectWalletButton.scss";
import connectWalletImg from "@/img/ic_wallet_24.svg";
import { Trans } from "@lingui/macro";
import { MdOutlineStopCircle } from "react-icons/md";
import cx from "classnames";

type Props = {
  small?: boolean,
  onConnect: () => void;
  onCancel?: () => void;
  connecting: boolean,
  disconnecting: boolean,
};

export default function ConnectWalletButton({
  small,
  onConnect,
  onCancel,
  connecting,
  disconnecting,
}: Props) {
  const [hover, setHover] = useState(false);

  const handleClick = useCallback(() => {
    if (!connecting && !disconnecting) {
      onConnect();
    } else if (connecting && !disconnecting) {
      if (onCancel) {
        onCancel();
      }
    }
  }, [connecting, disconnecting, onCancel, onConnect]);

  return (
    <button
      className={cx("connect-wallet-btn", {
        "connect-wallet-btn-connecting": connecting,
      })}
      onClick={handleClick}
      disabled={disconnecting}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
    >
      {!(connecting && hover) && <img className="btn-icon" src={connectWalletImg} alt="Connect Wallet" />}
      {(connecting && hover) && <MdOutlineStopCircle size={18} />}
      <span className="btn-label">{
        connecting ? <Trans>Connecting...</Trans> :
          disconnecting ? <Trans>Disconnecting...</Trans> :
            small ? <Trans>Connect</Trans> :
              <Trans>Connect Wallet</Trans>
      }</span>
    </button>
  );
}
