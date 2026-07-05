"use client";

import { useState } from "react";
import { PlusIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

type InlineCreateProps = {
  /** Singular entity noun, e.g. "board", "column", "card". */
  label: string;
  onCreate: (value: string) => void;
  placeholder?: string;
  maxLength?: number;
  className?: string;
};

function capitalize(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
}

/** Expandable "+ Add" affordance for board/column/card creation; Enter/Add submits, Esc cancels. */
export function InlineCreate({
  label,
  onCreate,
  placeholder,
  maxLength = 120,
  className,
}: InlineCreateProps) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | null>(null);

  function close() {
    setOpen(false);
    setDraft("");
    setError(null);
  }

  function submit() {
    const trimmed = draft.trim();
    if (trimmed.length === 0) {
      setError(`${capitalize(label)} name is required`);
      return;
    }
    if (trimmed.length > maxLength) {
      setError(`Name must be ${maxLength} characters or fewer`);
      return;
    }
    onCreate(trimmed);
    setDraft("");
    setError(null);
  }

  if (!open) {
    return (
      <Button
        variant="ghost"
        size="sm"
        onClick={() => setOpen(true)}
        className={cn("justify-start", className)}
      >
        <PlusIcon /> Add {label}
      </Button>
    );
  }

  return (
    <form
      className={cn("flex flex-col gap-1", className)}
      onSubmit={(event) => {
        event.preventDefault();
        submit();
      }}
    >
      <Input
        autoFocus
        aria-label={`New ${label} name`}
        aria-invalid={error ? true : undefined}
        placeholder={placeholder ?? `${capitalize(label)} name`}
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            close();
          }
        }}
      />
      {error && (
        <p role="alert" className="text-xs text-destructive">
          {error}
        </p>
      )}
      <div className="flex gap-1">
        <Button type="submit" size="sm">
          Add
        </Button>
        <Button type="button" size="sm" variant="ghost" onClick={close}>
          Cancel
        </Button>
      </div>
    </form>
  );
}
