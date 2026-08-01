import { useState } from "react";
import {
  Button,
  Group,
  Menu,
  SegmentedControl,
  Select,
  Stack,
  Title,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconChevronDown } from "@tabler/icons-react";
import { TaskBoard } from "../components/tasks/TaskBoard";
import { ProjectsView } from "../components/tasks/ProjectsView";
import { NewTaskModal } from "../components/tasks/NewTaskModal";
import { NewProjectModal } from "../components/tasks/NewProjectModal";
import { UNASSIGNED, type Project, type Task } from "../components/tasks/types";

// TODO: replace placeholders
const initialProjects: Project[] = [
  { id: "p1", title: "Website Redesign", description: "Marketing site refresh", color: "#7048e8", tags: [], isArchived: false },
  { id: "p2", title: "Home Reno", description: "Kitchen + bath", color: "#2f9e44", tags: [], isArchived: false },
];

const initialTasks: Task[] = [
  { id: "t1", title: "Design landing page", description: "", projectId: "p1", status: "todo", priority: "high", dueDate: new Date(), parentId: null, recurrence: null, tags: [] },
  { id: "t2", title: "Set up database schema", description: "", projectId: "p1", status: "in_progress", priority: "medium", dueDate: null, parentId: null, recurrence: null, tags: [] },
  { id: "t3", title: "Write unit tests", description: "", projectId: "p1", status: "todo", priority: "low", dueDate: null, parentId: null, recurrence: null, tags: [] },
  { id: "t4", title: "Ship v1", description: "", projectId: "p2", status: "done", priority: "urgent", dueDate: null, parentId: null, recurrence: null, tags: [] },
];

export function TasksPage() {
  const [view, setView] = useState("board");
  const [tasks, setTasks] = useState<Task[]>(initialTasks);
  const [projects, setProjects] = useState<Project[]>(initialProjects);
  const [projectId, setProjectId] = useState<string>(initialProjects[0].id);

  const [taskOpened, taskModal] = useDisclosure(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [projectOpened, projectModal] = useDisclosure(false);
  const [editingProject, setEditingProject] = useState<Project | null>(null);

  const activeProjects = projects.filter((p) => !p.isArchived);
  const projectOptions = [
    { value: UNASSIGNED, label: "Unassigned" },
    ...activeProjects.map((p) => ({ value: p.id, label: p.title })),
  ];
  const projectColor =
    projects.find((p) => p.id === projectId)?.color ?? undefined;

  function submitTask(task: Task) {
    setTasks((prev) =>
      prev.some((t) => t.id === task.id)
        ? prev.map((t) => (t.id === task.id ? task : t))
        : [...prev, task],
    );
  }
  const moveTask = (id: string, status: string) =>
    setTasks((prev) => prev.map((t) => (t.id === id ? { ...t, status } : t)));
  const deleteTask = (id: string) =>
    setTasks((prev) => prev.filter((t) => t.id !== id));
  const openNewTask = () => {
    setEditingTask(null);
    taskModal.open();
  };
  const openEditTask = (task: Task) => {
    setEditingTask(task);
    taskModal.open();
  };

  function submitProject(project: Project) {
    setProjects((prev) =>
      prev.some((p) => p.id === project.id)
        ? prev.map((p) => (p.id === project.id ? project : p))
        : [...prev, project],
    );
  }
  function archiveProject(id: string) {
    setProjects((prev) =>
      prev.map((p) => (p.id === id ? { ...p, isArchived: true } : p)),
    );
    if (projectId === id) setProjectId(UNASSIGNED);
  }
  function deleteProject(id: string) {
    setProjects((prev) => prev.filter((p) => p.id !== id));
    setTasks((prev) =>
      prev.map((t) => (t.projectId === id ? { ...t, projectId: UNASSIGNED } : t)),
    );
    if (projectId === id) setProjectId(UNASSIGNED);
  }
  const reorderProjects = (nextActive: Project[]) =>
    setProjects([...nextActive, ...projects.filter((p) => p.isArchived)]);
  const openProjectBoard = (id: string) => {
    setProjectId(id);
    setView("board");
  };
  const openNewProject = () => {
    setEditingProject(null);
    projectModal.open();
  };
  const openEditProject = (project: Project) => {
    setEditingProject(project);
    projectModal.open();
  };

  return (
    <Stack>
      <Title order={2}>Tasks</Title>

      <Group justify="space-between">
        <SegmentedControl
          value={view}
          onChange={setView}
          data={[
            { label: "Board", value: "board" },
            { label: "Projects", value: "projects" },
          ]}
        />

        <Group>
          {view === "board" && (
            <Select
              data={projectOptions}
              value={projectId}
              onChange={(value) => setProjectId(value ?? UNASSIGNED)}
              allowDeselect={false}
              aria-label="Select project"
              w={240}
            />
          )}

          <Menu position="bottom-end">
            <Menu.Target>
              <Button rightSection={<IconChevronDown size={16} stroke={1.5} />}>
                New
              </Button>
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Item onClick={openNewTask}>Task</Menu.Item>
              <Menu.Item onClick={openNewProject}>Project</Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </Group>
      </Group>

      {view === "board" ? (
        <TaskBoard
          tasks={tasks}
          projectId={projectId}
          projectColor={projectColor}
          onMoveTask={moveTask}
          onEditTask={openEditTask}
          onDeleteTask={deleteTask}
        />
      ) : (
        <ProjectsView
          projects={activeProjects}
          onReorder={reorderProjects}
          onOpen={openProjectBoard}
          onEdit={openEditProject}
          onArchive={archiveProject}
          onDelete={deleteProject}
        />
      )}

      <NewTaskModal
        opened={taskOpened}
        onClose={taskModal.close}
        projectOptions={projectOptions}
        defaultProjectId={projectId}
        initial={editingTask}
        onSubmit={submitTask}
      />
      <NewProjectModal
        opened={projectOpened}
        onClose={projectModal.close}
        initial={editingProject}
        onSubmit={submitProject}
      />
    </Stack>
  );
}
