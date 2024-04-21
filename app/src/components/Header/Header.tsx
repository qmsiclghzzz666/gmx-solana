import { useEffect, useState } from "react";
import cx from "classnames";
import { useMedia } from "react-use";
import { AnimatePresence, motion } from "framer-motion";
import { Link } from "react-router-dom";
import { RiMenuLine } from "react-icons/ri";
import { FaTimes } from "react-icons/fa";
import logoImg from "@/img/logo_GMSOL.png";

import "./Header.scss";
import { AppHeaderLinks } from "./AppHeaderLinks";
import { AppHeaderUser } from "./AppHeaderUser";

const FADE_VARIANTS = {
  hidden: { opacity: 0 },
  visible: { opacity: 1 },
};

const SLIDE_VARIANTS = {
  hidden: { x: "-100%" },
  visible: { x: 0 },
};

const TRANSITION = { duration: 0.2 };

export function Header() {
  const isMobile = useMedia("(max-width: 1200px)");

  const [isDrawerVisible, setIsDrawerVisible] = useState(false);
  const [isNativeSelectorModalVisible, setIsNativeSelectorModalVisible] = useState(false);

  useEffect(() => {
    if (isDrawerVisible) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "unset";
    }

    return () => {
      document.body.style.overflow = "unset";
    };
  }, [isDrawerVisible]);

  return (
    <>
      {isDrawerVisible && (
        <AnimatePresence>
          {isDrawerVisible && (
            <motion.div
              className="App-header-backdrop"
              initial="hidden"
              animate="visible"
              exit="hidden"
              variants={FADE_VARIANTS}
              transition={TRANSITION}
              onClick={() => setIsDrawerVisible(!isDrawerVisible)}
            ></motion.div>
          )}
        </AnimatePresence>
      )}
      {isNativeSelectorModalVisible && (
        <AnimatePresence>
          {isNativeSelectorModalVisible && (
            <motion.div
              className="selector-backdrop"
              initial="hidden"
              animate="visible"
              exit="hidden"
              variants={FADE_VARIANTS}
              transition={TRANSITION}
              onClick={() => setIsNativeSelectorModalVisible(!isNativeSelectorModalVisible)}
            ></motion.div>
          )}
        </AnimatePresence>
      )}
      <header>
        {!isMobile && (
          <div className="App-header large">
            <div className="App-header-container-left">
              <Link className="App-header-link-main" to="/">
                <img src={logoImg} height="21.462" className="big" alt="GMSOL Logo" />
                <img src={logoImg} className="small" alt="GMSOL Logo" />
              </Link>
              <AppHeaderLinks />
            </div>
            <div className="App-header-container-right">
              <AppHeaderUser />
            </div>
          </div>
        )}
        {isMobile && (
          <div className={cx("App-header", "small", { active: isDrawerVisible })}>
            <div
              className={cx("App-header-link-container", "App-header-top", {
                active: isDrawerVisible,
              })}
            >
              <div className="App-header-container-left">
                <div className="App-header-menu-icon-block" onClick={() => setIsDrawerVisible(!isDrawerVisible)}>
                  {!isDrawerVisible && <RiMenuLine className="App-header-menu-icon" />}
                  {isDrawerVisible && <FaTimes className="App-header-menu-icon" />}
                </div>
                <div className="App-header-link-main clickable" onClick={() => setIsDrawerVisible(!isDrawerVisible)}>
                  <img src={logoImg} height="21.462" className="big" alt="GMSOL Logo" />
                  <img src={logoImg} className="small" alt="GMSOL Logo" />
                </div>
              </div>
              <div className="App-header-container-right">
                <AppHeaderUser small />
              </div>
            </div>
          </div>
        )}
      </header>
      <AnimatePresence>
        {isDrawerVisible && (
          <motion.div
            onClick={() => setIsDrawerVisible(false)}
            className="App-header-links-container App-header-drawer"
            initial="hidden"
            animate="visible"
            exit="hidden"
            variants={SLIDE_VARIANTS}
            transition={TRANSITION}
          >
            <AppHeaderLinks
              small
              clickCloseIcon={() => setIsDrawerVisible(false)}
            />
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}
