import cx from "classnames";
import "./Footer.css";
import logoImg from "@/img/logo_GMSOL_footer.svg";
import { SOCIAL_LINKS, getFooterLinks } from "./utils";
import ExternalLink from "../ExternalLink/ExternalLink";
import { NavLink } from "react-router-dom";

export default function Footer() {
  return (
    <div className="Footer">
      <div className={cx("Footer-wrapper")}>
        <div className="Footer-logo">
          <img src={logoImg} />
        </div>
        <div className="Footer-social-link-block">
          {SOCIAL_LINKS.map((platform) => {
            return (
              <ExternalLink key={platform.name} className="App-social-link" href={platform.link}>
                <img src={platform.icon} alt={platform.name} />
              </ExternalLink>
            );
          })}
        </div>
        <div className="Footer-links">
          {getFooterLinks().map(({ external, label, link }) => {
            if (external) {
              return (
                <ExternalLink key={label} href={link} className="Footer-link">
                  {label}
                </ExternalLink>
              );
            }
            return (
              <NavLink key={link} to={link} className={({ isActive }) => {
                return `Footer-link ${isActive ? "active" : ""}`
              }}>
                {label}
              </NavLink>
            );
          })}
        </div>
      </div>
    </div>
  );
}
