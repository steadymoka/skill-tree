interface SectionProps {
  title: string;
  subtitle?: string;
  count?: number;
  children: React.ReactNode;
}

export function Section({ title, subtitle, count, children }: SectionProps) {
  return (
    <section>
      <h2 className="text-sm font-semibold text-muted-foreground mb-3">
        {title}
        {count !== undefined && (
          <span className="ml-2 text-foreground">({count})</span>
        )}
        {subtitle && (
          <span className="ml-2 font-normal text-xs">{subtitle}</span>
        )}
      </h2>
      {children}
    </section>
  );
}
