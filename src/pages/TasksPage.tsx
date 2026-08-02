import { useEffect, useMemo, useState } from "react";
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
import { commands, type Project as ApiProject, type Task as ApiTask } from "../bindings";
import { TaskBoard } from "../components/tasks/TaskBoard";
import { ProjectsView } from "../components/tasks/ProjectsView";
import { NewTaskModal } from "../components/tasks/NewTaskModal";
import { NewProjectModal } from "../components/tasks/NewProjectModal";
import {
  applyProjectView,
  applyTaskView,
  projectFromApi,
  taskFromApi,
} from "../components/tasks/mappers";
import { UNASSIGNED, type Project, type Task } from "../components/tasks/types";

export function TasksPage() {
  const [view, setView] = useState("board");
  const [tasks, setTasks] = useState<ApiTask[]>([]);
  const [projects, setProjects] = useState<ApiProject[]>([]);
  const [projectId, setProjectId] = useState<string>(UNASSIGNED);

  const [taskOpened, taskModal] = useDisclosure(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [projectOpened, projectModal] = useDisclosure(false);
  const [editingProject, setEditingProject] = useState<Project | null>(null);

  useEffect(() => {
    (async () => {
      const [taskRes, projectRes] = await Promise.all([
        commands.listTasks(),
        commands.listProjects(),
      ]);
      if (taskRes.status === "ok") setTasks(taskRes.data);
      else console.error("list_tasks failed", taskRes.error);
      if (projectRes.status === "ok") setProjects(projectRes.data);
      else console.error("list_projects failed", projectRes.error);
    })();
  }, []);

  const viewTasks = useMemo(() => tasks.map(taskFromApi), [tasks]);
  const viewProjects = useMemo(() => projects.map(projectFromApi), [projects]);

  const activeProjects = viewProjects.filter((p) => !p.isArchived);
  const projectOptions = [
    { value: UNASSIGNED, label: "Unassigned" },
    ...activeProjects.map((p) => ({ value: p.id, label: p.title })),
  ];
  const projectColor =
    viewProjects.find((p) => p.id === projectId)?.color ?? undefined;
  const parentOptions = viewTasks
    .filter((t) => t.id !== editingTask?.id)
    .map((t) => ({ value: t.id, label: t.title }));

  async function submitTask(taskView: Task) {
    const existing = tasks.find((t) => t.id === taskView.id);
    if (existing) {
      const updated = applyTaskView(existing, taskView);
      const res = await commands.saveTask(updated);
      if (res.status === "ok") {
        setTasks((prev) => prev.map((t) => (t.id === updated.id ? updated : t)));
      } else {
        console.error("save_task failed", res.error);
      }
      return;
    }
    const created = await commands.createTask(taskView.title, taskView.description);
    if (created.status !== "ok") {
      console.error("create_task failed", created.error);
      return;
    }
    const full = applyTaskView(created.data, taskView);
    const res = await commands.saveTask(full);
    if (res.status === "ok") {
      setTasks((prev) => [full, ...prev]);
    } else {
      console.error("save_task failed", res.error);
    }
  }

  async function moveTask(id: string, status: string) {
    const base = tasks.find((t) => t.id === id);
    if (!base) return;
    const updated: ApiTask = {
      ...base,
      status: status as ApiTask["status"],
      completed_at:
        status === "done"
          ? (base.completed_at ?? new Date().toISOString())
          : null,
    };
    setTasks((prev) => prev.map((t) => (t.id === id ? updated : t)));
    const res = await commands.saveTask(updated);
    if (res.status !== "ok") console.error("save_task failed", res.error);
  }

  async function deleteTask(id: string) {
    setTasks((prev) => prev.filter((t) => t.id !== id));
    const res = await commands.deleteTask(id);
    if (res.status !== "ok") console.error("delete_task failed", res.error);
  }

  const openNewTask = () => {
    setEditingTask(null);
    taskModal.open();
  };
  const openEditTask = (task: Task) => {
    setEditingTask(task);
    taskModal.open();
  };

  async function submitProject(projectView: Project) {
    const existing = projects.find((p) => p.id === projectView.id);
    if (existing) {
      const updated = applyProjectView(existing, projectView);
      const res = await commands.saveProject(updated);
      if (res.status === "ok") {
        setProjects((prev) =>
          prev.map((p) => (p.id === updated.id ? updated : p)),
        );
      } else {
        console.error("save_project failed", res.error);
      }
      return;
    }
    const created = await commands.createProject(
      projectView.title,
      projectView.description,
    );
    if (created.status !== "ok") {
      console.error("create_project failed", created.error);
      return;
    }
    const full = applyProjectView(created.data, projectView);
    const res = await commands.saveProject(full);
    if (res.status === "ok") {
      setProjects((prev) => [full, ...prev]);
    } else {
      console.error("save_project failed", res.error);
    }
  }

  async function archiveProject(id: string) {
    const base = projects.find((p) => p.id === id);
    if (!base) return;
    const updated: ApiProject = { ...base, is_archived: true };
    setProjects((prev) => prev.map((p) => (p.id === id ? updated : p)));
    if (projectId === id) setProjectId(UNASSIGNED);
    const res = await commands.saveProject(updated);
    if (res.status !== "ok") console.error("save_project failed", res.error);
  }

  async function deleteProject(id: string) {
    setProjects((prev) => prev.filter((p) => p.id !== id));
    setTasks((prev) =>
      prev.map((t) => (t.project_id === id ? { ...t, project_id: null } : t)),
    );
    if (projectId === id) setProjectId(UNASSIGNED);
    const res = await commands.deleteProject(id);
    if (res.status !== "ok") console.error("delete_project failed", res.error);
  }

  function reorderProjects(nextActive: Project[]) {
    const order = new Map(nextActive.map((p, index) => [p.id, index]));
    setProjects((prev) => {
      const active = prev
        .filter((p) => !p.is_archived)
        .sort((a, b) => (order.get(a.id) ?? 0) - (order.get(b.id) ?? 0));
      const archived = prev.filter((p) => p.is_archived);
      return [...active, ...archived];
    });
  }

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
          tasks={viewTasks}
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
        parentOptions={parentOptions}
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