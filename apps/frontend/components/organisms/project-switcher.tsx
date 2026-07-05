"use client";

import Link from "next/link";
import { ChevronsUpDownIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useProjects } from "@/lib/hooks/use-projects";

type ProjectSwitcherProps = {
  currentProjectId: string;
};

export function ProjectSwitcher({ currentProjectId }: ProjectSwitcherProps) {
  const { data: projects } = useProjects();
  const current = projects?.find((project) => project.id === currentProjectId);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger render={<Button variant="outline" size="sm" />}>
        {current?.name ?? "Select project"}
        <ChevronsUpDownIcon />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="min-w-48">
        {projects?.map((project) => (
          <DropdownMenuItem
            key={project.id}
            render={<Link href={`/projects/${project.id}`} />}
          >
            {project.name}
          </DropdownMenuItem>
        ))}
        <DropdownMenuSeparator />
        <DropdownMenuItem render={<Link href="/" />}>
          All projects
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
