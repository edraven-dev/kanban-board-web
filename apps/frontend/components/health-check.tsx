"use client";

import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";

const API_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:5000";
const DISPLAY_MS = 5000;

type Status =
  | { state: "hidden" }
  | { state: "loading" }
  | { state: "up"; value: string }
  | { state: "down"; message: string };

const INDICATOR = {
  loading: {
    dot: "bg-amber-500 animate-pulse",
    text: "text-amber-500",
    label: "Checking API…",
  },
  up: {
    dot: "bg-emerald-500",
    text: "text-emerald-500",
    label: "API is up",
  },
  down: {
    dot: "bg-red-500",
    text: "text-red-500",
    label: "API is down",
  },
} as const;

async function fetchHealth(): Promise<Status> {
  try {
    const res = await fetch(`${API_URL}/api/health`);
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}`);
    }
    const value = await res.text();
    return { state: "up", value: value.trim() };
  } catch (err) {
    return {
      state: "down",
      message: err instanceof Error ? err.message : "Request failed",
    };
  }
}

export function HealthCheck() {
  const [status, setStatus] = useState<Status>({ state: "hidden" });

  // Auto-hide the result 5s after it lands (resets if a new check comes in).
  useEffect(() => {
    if (status.state !== "up" && status.state !== "down") {
      return;
    }
    const timer = setTimeout(() => setStatus({ state: "hidden" }), DISPLAY_MS);
    return () => clearTimeout(timer);
  }, [status]);

  function handleClick() {
    if (status.state === "loading") {
      return;
    }
    setStatus({ state: "loading" });
    fetchHealth().then(setStatus);
  }

  const indicator = status.state === "hidden" ? null : INDICATOR[status.state];

  return (
    <div className="flex flex-col items-center gap-3 sm:items-start">
      <Button
        variant="outline"
        onClick={handleClick}
        aria-busy={status.state === "loading"}
        className={status.state === "loading" ? "opacity-70" : undefined}
      >
        {status.state === "loading" ? "Checking…" : "Check API health"}
      </Button>
      {indicator && (
        <div
          role="status"
          aria-live="polite"
          className="flex items-center gap-2.5 rounded-full border border-border bg-card px-4 py-2"
        >
          <span
            className={`inline-block size-2.5 rounded-full ${indicator.dot}`}
          />
          <span className={`text-sm font-medium ${indicator.text}`}>
            {indicator.label}
          </span>
          {status.state === "up" && (
            <span className="text-sm text-muted-foreground">
              ({status.value})
            </span>
          )}
          {status.state === "down" && (
            <span className="text-sm text-muted-foreground">
              ({status.message})
            </span>
          )}
        </div>
      )}
    </div>
  );
}
