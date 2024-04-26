import cx from "classnames";
import "./TokenIcon.scss";
import { getIconUrlPath } from "@/utils/icon";

type Props = {
  symbol: string;
  displaySize: number;
  importSize?: 24 | 40;
  className?: string;
};

function TokenIcon({ className, symbol, displaySize, importSize = 24 }: Props) {
  const iconPath = getIconUrlPath(symbol, importSize);
  const classNames = cx("Token-icon", className);
  if (!iconPath) return <></>;
  return <img className={classNames} src={iconPath} alt={symbol} width={displaySize} />;
}

export default TokenIcon;
