import { ReactNode } from "react";
import "./PageTitle.scss";
import cx from "classnames";
import icon_solana from "@/img/ic_solana_24.svg";

type Props = {
  title: string;
  subtitle?: string | ReactNode;
  className?: string;
  isTop?: boolean;
  showNetworkIcon?: boolean;
  afterTitle?: ReactNode;
};

export default function PageTitle({
  title,
  subtitle,
  className,
  isTop = false,
  showNetworkIcon = true,
  afterTitle,
}: Props) {
  const classNames = cx("Page-title-wrapper", className, { gapTop: !isTop });
  const currentNetworkIcon = icon_solana;
  return (
    <div className={classNames}>
      <div className="Page-title-group">
        <h2 className="Page-title__text">{title}</h2>
        {showNetworkIcon && <img className="Page-title__icon" src={currentNetworkIcon} alt="Current Network Icon" />}
        {afterTitle}
      </div>
      <div className="Page-subtitle-group">{subtitle}</div>
    </div>
  );
}
