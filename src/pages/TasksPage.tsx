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
import {
  commands,
  type Project as ApiProject,
  type RecurrenceRule as ApiRule,
  type Tag,
  type Task as ApiTask,
} from "../bindings";
import { TaskBoard } from "../components/tasks/TaskBoard";
import { ProjectsView } from "../components/tasks/ProjectsView";
import { NewTaskModal } from "../components/tasks/NewTaskModal";
import { NewProjectModal } from "../components/tasks/NewProjectModal";
import {
  applyProjectView,
  applyTaskView,
  projectFromApi,
  ruleFromDraft,
  taskFromApi,
} from "../components/tasks/mappers";
import { UNASSIGNED, type Project, type Task } from "../components/tasks/types";

export function TasksPage() {
  const [view, setView] = useState("board");
  const [tasks, setTasks] = useState<ApiTask[]>([]);
  const [projects, setProjects] = useState<ApiProject[]>([]);
  const [tags, setTags] = useState<Tag[]>([]);
  const [rules, setRules] = useState<ApiRule[]>([]);
  const [projectId, setProjectId] = useState<string>(UNASSIGNED);

  const [taskOpened, taskModal] = useDisclosure(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [projectOpened, projectModal] = useDisclosure(false);
  const [editingProject, setEditingProject] = useState<Project | null>(null);

  useEffect(() => {
    (async () => {
      const [taskRes, projectRes, tagRes, ruleRes] = await Promise.all([
        commands.listTasks(),
        commands.listProjects(),
        commands.listTags(),
        commands.listRecurrences(),
      ]);
      if (taskRes.status === "ok") setTasks(taskRes.data);
      else console.error("list_tasks failed", taskRes.error);
      if (projectRes.status === "ok") setProjects(projectRes.data);
      else console.error("list_projects failed", projectRes.error);
      if (tagRes.status === "ok") setTags(tagRes.data);
      else console.error("list_tags failed", tagRes.error);
      if (ruleRes.status === "ok") setRules(ruleRes.data);
      else console.error("list_recurrences failed", ruleRes.error);
    })();
  }, []);

  const rulesById = useMemo(
    () => new Map(rules.map((r) => [r.id, r])),
    [rules],
  );
  const viewTasks = useMemo(
    () => tasks.map((t) => taskFromApi(t, rulesById)),
    [tasks, rulesById],
  );
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
  const taskOptions = viewTasks.map((t) => ({ value: t.id, label: t.title }));
  const editingProjectTaskIds = editingProject
    ? tasks.filter((t) => t.project_id === editingProject.id).map((t) => t.id)
    : [];

  function resolveTags(labels: string[]): [Tag[], Tag[]] {
    const byLabel = new Map(tags.map((t) => [t.label, t]));
    const resolved: Tag[] = [];
    const created: Tag[] = [];
    const seen = new Set<string>();
    for (const raw of labels) {
      const label = raw.trim();
      if (!label || seen.has(label)) continue;
      seen.add(label);
      let tag = byLabel.get(label);
      if (!tag) {
        const now = new Date().toISOString();
        tag = { id: crypto.randomUUID(), created_at: now, updated_at: now, label, color: null };
        created.push(tag);
        byLabel.set(label, tag);
      }
      resolved.push(tag);
    }
    return [resolved, created];
  }

  async function submitTask(taskView: Task) {
    const existing = tasks.find((t) => t.id === taskView.id);
    const [tagRows, createdTags] = resolveTags(taskView.tags);

    let recurrenceId: string | null = existing?.recurrence_id ?? null;
    let savedRule: ApiRule | null = null;
    let ruleToDelete: string | null = null;
    if (taskView.recurrence) {
      const baseRule =
        (existing?.recurrence_id && rulesById.get(existing.recurrence_id)) ||
        null;
      let ruleBase = baseRule;
      if (!ruleBase) {
        const created = await commands.createRecurrence();
        if (created.status !== "ok") {
          console.error("create_recurrence failed", created.error);
          return;
        }
        ruleBase = created.data;
      }
      savedRule = ruleFromDraft(ruleBase, taskView.recurrence);
      const res = await commands.saveRecurrence(savedRule);
      if (res.status !== "ok") {
        console.error("save_recurrence failed", res.error);
        return;
      }
      recurrenceId = savedRule.id;
    } else if (existing?.recurrence_id) {
      recurrenceId = null;
      ruleToDelete = existing.recurrence_id;
    }

    let base = existing;
    if (!base) {
      const created = await commands.createTask(
        taskView.title,
        taskView.description,
      );
      if (created.status !== "ok") {
        console.error("create_task failed", created.error);
        return;
      }
      base = created.data;
    }
    const full: ApiTask = {
      ...applyTaskView(base, taskView, tagRows),
      recurrence_id: recurrenceId,
    };
    const res = await commands.saveTask(full);
    if (res.status !== "ok") {
      console.error("save_task failed", res.error);
      return;
    }

    if (ruleToDelete) {
      const del = await commands.deleteRecurrence(ruleToDelete);
      if (del.status !== "ok")
        console.error("delete_recurrence failed", del.error);
    }

    if (createdTags.length) setTags((prev) => [...prev, ...createdTags]);
    setRules((prev) => {
      let next = prev;
      if (savedRule) {
        next = prev.some((r) => r.id === savedRule!.id)
          ? prev.map((r) => (r.id === savedRule!.id ? savedRule! : r))
          : [savedRule!, ...prev];
      }
      if (ruleToDelete) next = next.filter((r) => r.id !== ruleToDelete);
      return next;
    });
    setTasks((prev) =>
      existing
        ? prev.map((t) => (t.id === full.id ? full : t))
        : [full, ...prev],
    );
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

  async function submitProject(projectView: Project, taskIds: string[]) {
    const existing = projects.find((p) => p.id === projectView.id);
    const [tagRows, createdTags] = resolveTags(projectView.tags);

    let base = existing;
    if (!base) {
      const created = await commands.createProject(
        projectView.title,
        projectView.description,
      );
      if (created.status !== "ok") {
        console.error("create_project failed", created.error);
        return;
      }
      base = created.data;
    }
    const full = applyProjectView(base, projectView, tagRows);
    const res = await commands.saveProject(full);
    if (res.status !== "ok") {
      console.error("save_project failed", res.error);
      return;
    }

    const selected = new Set(taskIds);
    const changed = tasks
      .filter((t) => (t.project_id === full.id) !== selected.has(t.id))
      .map((t) => ({
        ...t,
        project_id: selected.has(t.id) ? full.id : null,
      }));
    for (const task of changed) {
      const r = await commands.saveTask(task);
      if (r.status !== "ok") console.error("save_task failed", r.error);
    }

    if (createdTags.length) setTags((prev) => [...prev, ...createdTags]);
    setProjects((prev) =>
      existing ? prev.map((p) => (p.id === full.id ? full : p)) : [full, ...prev],
    );
    if (changed.length) {
      const changedById = new Map(changed.map((t) => [t.id, t]));
      setTasks((prev) => prev.map((t) => changedById.get(t.id) ?? t));
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
        taskOptions={taskOptions}
        initialTaskIds={editingProjectTaskIds}
        onSubmit={submitProject}
      />
    </Stack>
  );
}