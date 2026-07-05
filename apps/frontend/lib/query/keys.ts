export const queryKeys = {
  projects: ["projects"] as const,
  boardsByProject: (projectId: string) =>
    ["projects", projectId, "boards"] as const,
  boardFull: (boardId: string) => ["boards", boardId, "full"] as const,
  columnsByBoard: (boardId: string) => ["boards", boardId, "columns"] as const,
  cardsByColumn: (columnId: string) => ["columns", columnId, "cards"] as const,
  card: (cardId: string) => ["cards", cardId] as const,
};
