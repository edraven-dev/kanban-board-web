import { LoaderCircleIcon } from "lucide-react";

import { cn } from "@/lib/utils";

type SpinnerProps = {
  label?: string;
  className?: string;
};

/** Accessible loading indicator. */
export function Spinner({ label = "Loading…", className }: SpinnerProps) {
  return (
    <span role="status" className={cn("inline-flex", className)}>
      <LoaderCircleIcon
        aria-hidden
        className="size-4 animate-spin text-muted-foreground"
      />
      <span className="sr-only">{label}</span>
    </span>
  );
}
