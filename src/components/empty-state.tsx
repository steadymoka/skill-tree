import { Card, CardContent } from "@/components/ui/card";

export function EmptyState({ message }: { message: string }) {
  return (
    <Card>
      <CardContent className="flex items-center justify-center py-12">
        <p className="text-muted-foreground text-sm">{message}</p>
      </CardContent>
    </Card>
  );
}
