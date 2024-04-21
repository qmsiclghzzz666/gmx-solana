import cx from "classnames";
import "./TokenIcon.scss";

function getIconUrlPath(symbol: string, size: 24 | 40) {
  if (!symbol || !size) return;
  return `src/img/ic_${symbol.toLowerCase()}_${size}.svg`;
}

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
