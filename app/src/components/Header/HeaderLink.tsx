import { MouseEventHandler, ReactNode } from "react";
import { NavLink } from "react-router-dom";
import cx from "classnames";

import "./Header.scss";

interface Props {
  isHomeLink?: boolean;
  className?: string;
  exact?: boolean;
  to: string;
  onClick?: MouseEventHandler<HTMLDivElement | HTMLAnchorElement>;
  children?: ReactNode;
}

export function HeaderLink({
  isHomeLink,
  className,
  to,
  children,
  onClick,
}: Props) {

  if (isHomeLink) {
    return (
      <a href="/" className={cx(className)} onClick={onClick}>
        {children}
      </a>
    );
  }

  return (
    <NavLink
      className={({ isActive }) =>
        `${className} ${isActive ? "active" : ""}`
      }
      to={to}
      onClick={onClick}
    >
      {children}
    </NavLink>
  );
}
