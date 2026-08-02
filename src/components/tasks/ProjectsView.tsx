import {
  DndContext,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  arrayMove,
  rectSortingStrategy,
} from "@dnd-kit/sortable";
import { SimpleGrid, Text } from "@mantine/core";
import { ProjectCard } from "./ProjectCard";
import type { Project } from "./types";

interface ProjectsViewProps {
  projects: Project[];
  onReorder: (projects: Project[]) => void;
  onOpen: (id: string) => void;
  onEdit: (project: Project) => void;
  onArchive: (id: string) => void;
  onDelete: (id: string) => void;
}

export function ProjectsView({
  projects,
  onReorder,
  onOpen,
  onEdit,
  onArchive,
  onDelete,
}: ProjectsViewProps) {
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
  );

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    if (!over || active.id === over.id) return;
    const oldIndex = projects.findIndex((p) => p.id === active.id);
    const newIndex = projects.findIndex((p) => p.id === over.id);
    onReorder(arrayMove(projects, oldIndex, newIndex));
  }

  if (projects.length === 0) {
    return (
      <Text c="dimmed">No projects yet.</Text>
    );
  }

  return (
    <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
      <SortableContext
        items={projects.map((p) => p.id)}
        strategy={rectSortingStrategy}
      >
        <SimpleGrid cols={{ base: 1, sm: 2, md: 3 }}>
          {projects.map((project) => (
            <ProjectCard
              key={project.id}
              project={project}
              onOpen={onOpen}
              onEdit={onEdit}
              onArchive={onArchive}
              onDelete={onDelete}
            />
          ))}
        </SimpleGrid>
      </SortableContext>
    </DndContext>
  );
}
