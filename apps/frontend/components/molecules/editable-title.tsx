"use client";

import { useState } from "react";

import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

type EditableTitleProps = {
  value: string;
  onSave: (value: string) => void;
  /** Accessible name for the field, also used in validation messages. */
  label: string;
  maxLength?: number;
  className?: string;
};

/**
 * Inline-editable label reused for project/board/column names. Click to edit;
 * Enter or blur saves, Esc cancels. Empty/too-long values are rejected with a
 * message instead of saving.
 */
export function EditableTitle({
  value,
  onSave,
  label,
  maxLength = 120,
  className,
}: EditableTitleProps) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);
  const [error, setError] = useState<string | null>(null);

  function startEditing() {
    setDraft(value);
    setError(null);
    setEditing(true);
  }

  function validate(next: string): string | null {
    if (next.length === 0) return `${label} is required`;
    if (next.length > maxLength)
      return `${label} must be ${maxLength} characters or fewer`;
    return null;
  }

  function save() {
    const trimmed = draft.trim();
    const validation = validate(trimmed);
    if (validation) {
      setError(validation);
      return;
    }
    if (trimmed !== value) onSave(trimmed);
    setError(null);
    setEditing(false);
  }

  function cancel() {
    setError(null);
    setEditing(false);
  }

  if (!editing) {
    return (
      <button
        type="button"
        onClick={startEditing}
        className={cn("text-left", className)}
      >
        {value}
      </button>
    );
  }

  return (
    <div className={cn("flex flex-col gap-1", className)}>
      <Input
        autoFocus
        aria-label={label}
        aria-invalid={error ? true : undefined}
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            save();
          } else if (event.key === "Escape") {
            event.preventDefault();
            cancel();
          }
        }}
        onBlur={save}
      />
      {error && (
        <p role="alert" className="text-xs text-destructive">
          {error}
        </p>
      )}
    </div>
  );
}
