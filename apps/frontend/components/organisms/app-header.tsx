import Link from "next/link";

export function AppHeader() {
  return (
    <header className="flex h-14 shrink-0 items-center justify-between border-b border-border px-4 sm:px-6">
      <Link href="/" className="text-base font-semibold tracking-tight">
        Kanban
      </Link>
    </header>
  );
}
