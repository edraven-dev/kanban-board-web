import { BoardCanvas } from "@/components/organisms/board-canvas";

export default async function BoardPage({
  params,
}: {
  params: Promise<{ boardId: string }>;
}) {
  const { boardId } = await params;

  return <BoardCanvas boardId={boardId} />;
}
