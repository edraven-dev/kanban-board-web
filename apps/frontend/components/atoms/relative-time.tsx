"use client";

import { useEffect, useState } from "react";
import { formatDistanceToNow } from "date-fns";

type RelativeTimeProps = {
  date: string | Date;
  /** Text placed before the humanized distance. */
  prefix?: string;
  className?: string;
};

/** Humanized, self-refreshing relative time (re-renders every 60s). */
export function RelativeTime({
  date,
  prefix = "created ",
  className,
}: RelativeTimeProps) {
  const target = typeof date === "string" ? new Date(date) : date;
  const [, setTick] = useState(0);

  useEffect(() => {
    const id = setInterval(() => setTick((t) => t + 1), 60_000);
    return () => clearInterval(id);
  }, []);

  return (
    <time dateTime={target.toISOString()} className={className}>
      {prefix}
      {formatDistanceToNow(target, { addSuffix: true })}
    </time>
  );
}
