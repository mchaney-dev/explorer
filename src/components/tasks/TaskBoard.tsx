import {
  DndContext,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import { Group } from "@mantine/core";
import { TaskColumn } from "./TaskColumn";
import type { Task } from "./types";

const columns = [
  { status: "todo", label: "To Do" },
  { status: "in_progress", label: "In Progress" },
  { status: "done", label: "Done" },
];

interface TaskBoardProps {
  tasks: Task[];
  projectId: string;
  projectColor?: string;
  onMoveTask: (id: string, status: string) => void;
  onEditTask: (task: Task) => void;
  onDeleteTask: (id: string) => void;
}

export function TaskBoard({
  tasks,
  projectId,
  projectColor,
  onMoveTask,
  onEditTask,
  onDeleteTask,
}: TaskBoardProps) {
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
  );

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    if (!over) return;
    onMoveTask(String(active.id), String(over.id));
  }

  const projectTasks = tasks.filter((task) => task.projectId === projectId);

  return (
    <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
      <Group grow align="flex-start">
        {columns.map((column) => (
          <TaskColumn
            key={column.status}
            status={column.status}
            title={column.label}
            tasks={projectTasks.filter((task) => task.status === column.status)}
            color={projectColor}
            onEditTask={onEditTask}
            onDeleteTask={onDeleteTask}
          />
        ))}
      </Group>
    </DndContext>
  );
}
