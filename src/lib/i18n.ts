"use client";

import { createContext, useContext } from "react";
import { ko, type Messages } from "@/messages/ko";
import { en } from "@/messages/en";

export type Locale = "ko" | "en";
export type { Messages };

export const messages: Record<Locale, Messages> = { ko, en };

interface LocaleContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: Messages;
}

export const LocaleContext = createContext<LocaleContextValue>({
  locale: "ko",
  setLocale: () => {},
  t: ko,
});

export function useT(): Messages {
  return useContext(LocaleContext).t;
}

export function useLocale() {
  return useContext(LocaleContext);
}

export function getMessages(locale: Locale): Messages {
  return messages[locale];
}
