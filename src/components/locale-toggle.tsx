"use client";

import { useLocale } from "@/lib/i18n";
import { Button } from "@/components/ui/button";

export function LocaleToggle() {
  const { locale, setLocale } = useLocale();

  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={() => setLocale(locale === "ko" ? "en" : "ko")}
      className="text-xs font-mono"
    >
      {locale === "ko" ? "EN" : "KO"}
    </Button>
  );
}
