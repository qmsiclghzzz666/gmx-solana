import cx from "classnames";
import "./Footer.css";
import logoImg from "@/img/logo_GMSOL.png";

export default function Footer() {
  return (
    <div className="Footer">
      <div className={cx("Footer-wrapper")}>
        <div className="Footer-logo">
          <img src={logoImg} />
        </div>
        <div className="Footer-social-link-block">
        </div>
        <div className="Footer-links">
        </div>
      </div>
    </div>
  );
}
