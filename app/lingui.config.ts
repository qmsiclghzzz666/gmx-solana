import { LinguiConfig } from "@lingui/conf";

const config: LinguiConfig = {
  locales: [
    "en",
    "es",
    "ko",
    "ja",
    "zh",
    "ru",
    "fr",
    "de",
    "pseudo"
  ],
  sourceLocale: "en",
  catalogs: [
    {
      path: "<rootDir>/src/locales/{locale}/messages",
      include: ["src"],
    },
  ],
  formatOptions: {
    lineNumbers: false
  },
  fallbackLocales: {
    default: "en"
  },
  format: "po",
  orderBy: "messageId",
  pseudoLocale: "pseudo"
};

export default config;
