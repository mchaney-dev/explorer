import { useDroppable } from "@dnd-kit/core";
import { Paper, Stack, Text } from "@mantine/core";
import { TaskCard } from "./TaskCard";
import type { Task } from "./types";

interface TaskColumnProps {
  status: string;
  title: string;
  tasks: Task[];
  color?: string;
  onEditTask: (task: Task) => void;
  onDeleteTask: (id: string) => void;
}

export function TaskColumn({
  status,
  title,
  tasks,
  color,
  onEditTask,
  onDeleteTask,
}: TaskColumnProps) {
  const { setNodeRef, isOver } = useDroppable({ id: status });

  return (
    <Paper
      ref={setNodeRef}
      withBorder
      p="md"
      mih={400}
      bg={isOver ? "var(--mantine-color-default-hover)" : undefined}
    >
      <Stack gap="sm">
        <Text fw={600} ta="center">
          {title}
        </Text>

        {tasks.map((task) => (
          <TaskCard
            key={task.id}
            task={task}
            color={color}
            onEdit={onEditTask}
            onDelete={onDeleteTask}
          />
        ))}

        {tasks.length === 0 && (
          <Text size="sm" c="dimmed" ta="center">
            Drop tasks here
          </Text>
        )}
      </Stack>
    </Paper>
  );
}
