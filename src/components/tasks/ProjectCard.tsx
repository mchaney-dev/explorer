import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import {
  ActionIcon,
  Badge,
  ColorSwatch,
  Group,
  Menu,
  Paper,
  Stack,
  Text,
} from "@mantine/core";
import { IconDots } from "@tabler/icons-react";
import type { Project } from "./types";

interface ProjectCardProps {
  project: Project;
  onOpen: (id: string) => void;
  onEdit: (project: Project) => void;
  onArchive: (id: string) => void;
  onDelete: (id: string) => void;
}

export function ProjectCard({
  project,
  onOpen,
  onEdit,
  onArchive,
  onDelete,
}: ProjectCardProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } =
    useSortable({ id: project.id });

  return (
    <Paper
      ref={setNodeRef}
      withBorder
      p="md"
      mih={120}
      onClick={() => onOpen(project.id)}
      {...listeners}
      {...attributes}
      style={{
        transform: CSS.Transform.toString(transform),
        transition,
        zIndex: isDragging ? 100 : undefined,
        opacity: isDragging ? 0.85 : 1,
        cursor: "grab",
        borderLeft: project.color ? `4px solid ${project.color}` : undefined,
      }}
    >
      <Stack gap="xs">
        <Group justify="space-between" wrap="nowrap">
          <Group gap="xs">
            {project.color && <ColorSwatch color={project.color} size={14} />}
            <Text fw={600}>{project.title}</Text>
          </Group>

          <Menu position="bottom-end">
            <Menu.Target>
              <ActionIcon
                variant="subtle"
                color="gray"
                size="sm"
                aria-label="Project actions"
                onPointerDown={(e) => e.stopPropagation()}
                onClick={(e) => e.stopPropagation()}
              >
                <IconDots size={16} />
              </ActionIcon>
            </Menu.Target>
            <Menu.Dropdown onClick={(e) => e.stopPropagation()}>
              <Menu.Item onClick={() => onEdit(project)}>Edit</Menu.Item>
              <Menu.Item onClick={() => onArchive(project.id)}>Archive</Menu.Item>
              <Menu.Item color="red" onClick={() => onDelete(project.id)}>
                Delete
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </Group>

        <Text size="sm" c="dimmed">
          {project.description || "No description"}
        </Text>

        {project.tags.length > 0 && (
          <Group gap={4}>
            {project.tags.map((tag) => (
              <Badge key={tag} size="xs" variant="light" color="gray">
                {tag}
              </Badge>
            ))}
          </Group>
        )}
      </Stack>
    </Paper>
  );
}
