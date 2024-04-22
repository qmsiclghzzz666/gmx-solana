import { t } from "@lingui/macro";
import "./Footer.css";
import xIcon from "@/img/ic_x.svg";
import telegramIcon from "@/img/ic_telegram.svg";
import githubIcon from "@/img/ic_github.svg";

type Link = {
  label: string;
  link: string;
  external?: boolean;
  isAppLink?: boolean;
};

type SocialLink = {
  link: string;
  name: string;
  icon: string;
};

export function getFooterLinks() {
  const FOOTER_LINKS: { app: Link[] } = {
    app: [
      { label: t`Media Kit`, link: "#", external: true },
    ],
  };
  return FOOTER_LINKS["app"];
}

export const SOCIAL_LINKS: SocialLink[] = [
  { link: "#", name: "Twitter", icon: xIcon },
  { link: "#", name: "Github", icon: githubIcon },
  { link: "#", name: "Telegram", icon: telegramIcon },
];
