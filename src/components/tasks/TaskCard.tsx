import { useDraggable } from "@dnd-kit/core";
import { ActionIcon, Badge, Card, Group, Menu, Text } from "@mantine/core";
import { IconDots, IconRepeat } from "@tabler/icons-react";
import type { Task } from "./types";

const priorityColors: Record<string, string> = {
  low: "gray",
  medium: "blue",
  high: "orange",
  urgent: "red",
};

interface TaskCardProps {
  task: Task;
  color?: string;
  onEdit: (task: Task) => void;
  onDelete: (id: string) => void;
}

export function TaskCard({ task, color, onEdit, onDelete }: TaskCardProps) {
  const { attributes, listeners, setNodeRef, transform, isDragging } =
    useDraggable({ id: task.id });

  return (
    <Card
      ref={setNodeRef}
      withBorder
      padding="sm"
      shadow={isDragging ? "md" : "xs"}
      onClick={() => onEdit(task)}
      {...listeners}
      {...attributes}
      style={{
        transform: transform
          ? `translate(${transform.x}px, ${transform.y}px)`
          : undefined,
        zIndex: isDragging ? 100 : undefined,
        opacity: isDragging ? 0.85 : 1,
        cursor: "grab",
        borderLeft: color ? `4px solid ${color}` : undefined,
      }}
    >
      <Group justify="space-between" wrap="nowrap" gap="xs">
        <Text size="sm" fw={500} lineClamp={2}>
          {task.title}
        </Text>

        <Group gap={4} wrap="nowrap">
          <Badge size="xs" color={priorityColors[task.priority] ?? "gray"}>
            {task.priority}
          </Badge>

          <Menu position="bottom-end">
            <Menu.Target>
              <ActionIcon
                variant="subtle"
                color="gray"
                size="sm"
                aria-label="Task actions"
                onPointerDown={(e) => e.stopPropagation()}
                onClick={(e) => e.stopPropagation()}
              >
                <IconDots size={16} />
              </ActionIcon>
            </Menu.Target>
            <Menu.Dropdown onClick={(e) => e.stopPropagation()}>
              <Menu.Item onClick={() => onEdit(task)}>Edit</Menu.Item>
              <Menu.Item color="red" onClick={() => onDelete(task.id)}>
                Delete
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </Group>
      </Group>

      {(task.dueDate || task.recurrence) && (
        <Group gap="sm" mt={6} wrap="nowrap">
          {task.dueDate && (
            <Text size="xs" c="dimmed">
              Due {task.dueDate.toLocaleDateString()}
            </Text>
          )}
          {task.recurrence && (
            <Group gap={4} wrap="nowrap" c="dimmed">
              <IconRepeat size={12} stroke={1.5} />
              <Text size="xs" tt="capitalize">
                {task.recurrence.frequency}
              </Text>
            </Group>
          )}
        </Group>
      )}

      {task.tags.length > 0 && (
        <Group gap={4} mt={6}>
          {task.tags.map((tag) => (
            <Badge key={tag} size="xs" variant="light" color="gray">
              {tag}
            </Badge>
          ))}
        </Group>
      )}
    </Card>
  );
}
