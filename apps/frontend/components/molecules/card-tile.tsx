import { RelativeTime } from "@/components/atoms/relative-time";
import { Card, CardContent } from "@/components/ui/card";
import type { Card as CardModel } from "@/lib/api/schemas";
import { cn } from "@/lib/utils";

/** Compact card: title and humanized created time only. */
export function CardTile({
  card,
  className,
}: {
  card: CardModel;
  className?: string;
}) {
  return (
    <Card size="sm" className={cn("cursor-grab", className)}>
      <CardContent className="flex flex-col gap-1">
        <span className="truncate text-sm font-medium">{card.title}</span>
        <RelativeTime
          date={card.createdAt}
          className="text-xs text-muted-foreground"
        />
      </CardContent>
    </Card>
  );
}
