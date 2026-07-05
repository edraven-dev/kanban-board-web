import { apiFetch } from "./client";
import {
  cardListSchema,
  cardSchema,
  type Card,
  type CreateCardInput,
  type MoveCardInput,
  type UpdateCardInput,
} from "./schemas";

export async function listCards(columnId: string): Promise<Card[]> {
  return cardListSchema.parse(await apiFetch(`/columns/${columnId}/cards`));
}

export async function getCard(cardId: string): Promise<Card> {
  return cardSchema.parse(await apiFetch(`/cards/${cardId}`));
}

export async function createCard(
  columnId: string,
  input: CreateCardInput,
): Promise<Card> {
  return cardSchema.parse(
    await apiFetch(`/columns/${columnId}/cards`, {
      method: "POST",
      body: input,
    }),
  );
}

export async function updateCard(
  cardId: string,
  input: UpdateCardInput,
): Promise<Card> {
  return cardSchema.parse(
    await apiFetch(`/cards/${cardId}`, { method: "PATCH", body: input }),
  );
}

export async function deleteCard(cardId: string): Promise<void> {
  await apiFetch(`/cards/${cardId}`, { method: "DELETE" });
}

export async function moveCard(
  cardId: string,
  input: MoveCardInput,
): Promise<void> {
  await apiFetch(`/cards/${cardId}/move`, { method: "PUT", body: input });
}
