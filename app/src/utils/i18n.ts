import { IS_DEVELOPMENT } from "@/config/env";
import { LANGUAGE_LOCALSTORAGE_KEY } from "@/config/localStorage";
import { Messages, i18n } from "@lingui/core";

export const locales = {
  en: "English",
  es: "Spanish",
  zh: "Chinese",
  ko: "Korean",
  ru: "Russian",
  ja: "Japanese",
  fr: "French",
  de: "German",
  ...(IS_DEVELOPMENT && { pseudo: "Test" }),
};

export const defaultLocale = "en";

export function isTestLanguage(locale: string) {
  return locale === "pseudo";
}

export async function dynamicActivate(locale: string) {
  const { messages } = (await import(`../locales/${locale}/messages.po`) as { messages: Messages });
  if (!isTestLanguage(locale)) {
    localStorage.setItem(LANGUAGE_LOCALSTORAGE_KEY, locale);
  }
  i18n.loadAndActivate({ locale, messages: messages });
}
