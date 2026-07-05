import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

/**
 * Responsive tile grid — column count scales with viewport width. Reused for
 * the project chooser and, later, a project's board list.
 */
export function TileGrid({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6",
        className,
      )}
    >
      {children}
    </div>
  );
}
