"use client";

import { cn } from "@/lib/utils";

interface PillToggleProps {
  isActive: boolean;
  onClick?: () => void;
  children: React.ReactNode;
  className?: string;
}

export function PillToggle({
  isActive,
  onClick,
  children,
  className,
}: PillToggleProps) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "rounded-md px-3 py-1.5 text-sm transition-colors",
        isActive
          ? "bg-accent text-accent-foreground font-medium"
          : "text-muted-foreground hover:bg-accent/50",
        className
      )}
    >
      {children}
    </button>
  );
}
