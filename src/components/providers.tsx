"use client";

import { useState, type ReactNode } from "react";
import { ThemeProvider } from "next-themes";
import { LocaleContext, type Locale, messages } from "@/lib/i18n";

export function Providers({ children }: { children: ReactNode }) {
  const [locale, setLocale] = useState<Locale>("ko");

  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false}>
      <LocaleContext.Provider
        value={{ locale, setLocale, t: messages[locale] }}
      >
        {children}
      </LocaleContext.Provider>
    </ThemeProvider>
  );
}
