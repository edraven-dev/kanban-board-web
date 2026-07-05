"use client";

import { useState } from "react";
import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { useForm } from "react-hook-form";
import { z } from "zod";

import { RelativeTime } from "@/components/atoms/relative-time";
import { ConfirmDeleteDialog } from "@/components/molecules/confirm-delete-dialog";
import { EditableTitle } from "@/components/molecules/editable-title";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Textarea } from "@/components/ui/textarea";
import type { Card } from "@/lib/api/schemas";
import { useDeleteCard, useUpdateCard } from "@/lib/hooks/use-cards";

const descriptionSchema = z.object({ description: z.string().max(10000) });
type DescriptionValues = z.infer<typeof descriptionSchema>;

type CardModalProps = {
  boardId: string;
  card: Card | undefined;
  columnName: string | undefined;
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

export function CardModal({
  boardId,
  card,
  columnName,
  open,
  onOpenChange,
}: CardModalProps) {
  const update = useUpdateCard(boardId);
  const remove = useDeleteCard(boardId);
  const [confirmOpen, setConfirmOpen] = useState(false);

  return (
    <>
      <Dialog open={open} onOpenChange={(next) => onOpenChange(next)}>
        {card && (
          <DialogContent className="sm:max-w-md">
            <DialogHeader>
              <DialogTitle>
                <EditableTitle
                  value={card.title}
                  label="Card title"
                  maxLength={200}
                  onSave={(title) => update.mutate({ id: card.id, title })}
                />
              </DialogTitle>
            </DialogHeader>

            <DescriptionForm
              key={card.id}
              description={card.description}
              onSave={(description) =>
                update.mutate({ id: card.id, description })
              }
            />

            <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs text-muted-foreground">
              <dt>Column</dt>
              <dd className="text-foreground">{columnName}</dd>
              <dt>Created</dt>
              <dd>
                <RelativeTime date={card.createdAt} />
              </dd>
              <dt>Updated</dt>
              <dd>
                <RelativeTime date={card.updatedAt} prefix="updated " />
              </dd>
              <dt>ID</dt>
              <dd className="truncate font-mono">{card.id}</dd>
            </dl>

            <Button
              variant="destructive"
              size="sm"
              className="self-start"
              onClick={() => setConfirmOpen(true)}
            >
              Delete card
            </Button>
          </DialogContent>
        )}
      </Dialog>

      <ConfirmDeleteDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        title="Delete this card?"
        description="This permanently deletes the card."
        onConfirm={() => {
          if (card) remove.mutate(card.id);
          onOpenChange(false);
        }}
      />
    </>
  );
}

function DescriptionForm({
  description,
  onSave,
}: {
  description: string;
  onSave: (description: string) => void;
}) {
  const form = useForm<DescriptionValues>({
    resolver: standardSchemaResolver(descriptionSchema),
    defaultValues: { description },
  });

  return (
    <form
      onSubmit={form.handleSubmit((values) => {
        onSave(values.description);
        form.reset(values);
      })}
      className="flex flex-col gap-2"
    >
      <label
        htmlFor="card-description"
        className="text-xs font-medium text-muted-foreground"
      >
        Description
      </label>
      <Textarea
        id="card-description"
        placeholder="Add a description…"
        {...form.register("description")}
      />
      {form.formState.errors.description && (
        <p role="alert" className="text-xs text-destructive">
          {form.formState.errors.description.message}
        </p>
      )}
      <Button
        type="submit"
        size="sm"
        className="self-start"
        disabled={!form.formState.isDirty}
      >
        Save description
      </Button>
    </form>
  );
}
