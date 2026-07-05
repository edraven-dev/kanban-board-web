import { ProjectSwitcher } from "@/components/organisms/project-switcher";

export default async function ProjectPage({
  params,
}: {
  params: Promise<{ projectId: string }>;
}) {
  const { projectId } = await params;

  return (
    <div className="flex flex-1 flex-col">
      <header className="flex items-center gap-3 border-b border-border px-6 py-3">
        <ProjectSwitcher currentProjectId={projectId} />
      </header>
      <main className="flex flex-1 flex-col gap-4 px-6 py-8">
        <p className="text-sm text-muted-foreground">
          Boards for this project will appear here.
        </p>
      </main>
    </div>
  );
}
