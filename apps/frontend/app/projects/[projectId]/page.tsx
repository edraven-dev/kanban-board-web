import { ProjectBoards } from "@/components/organisms/project-boards";

export default async function ProjectPage({
  params,
}: {
  params: Promise<{ projectId: string }>;
}) {
  const { projectId } = await params;

  return <ProjectBoards projectId={projectId} />;
}
