import type {
  Priority,
  Project as ApiProject,
  Status,
  Task as ApiTask,
} from "../../bindings";
import { UNASSIGNED, type Project, type Task } from "./types";

export function taskFromApi(t: ApiTask): Task {
  return {
    id: t.id,
    title: t.title,
    description: t.description,
    projectId: t.project_id ?? UNASSIGNED,
    status: t.status ?? "todo",
    priority: t.priority ?? "medium",
    dueDate: t.due_date ? new Date(t.due_date) : null,
    parentId: t.parent_id,
    recurrence: null,
    tags: [],
  };
}

export function applyTaskView(base: ApiTask, v: Task): ApiTask {
  const status = v.status as Status;
  return {
    ...base,
    title: v.title,
    description: v.description,
    project_id: v.projectId === UNASSIGNED ? null : v.projectId,
    parent_id: v.parentId,
    status,
    priority: v.priority as Priority,
    due_date: v.dueDate ? v.dueDate.toISOString() : null,
    completed_at:
      status === "done" ? (base.completed_at ?? new Date().toISOString()) : null,
  };
}

export function projectFromApi(p: ApiProject): Project {
  return {
    id: p.id,
    title: p.title,
    description: p.description,
    color: p.color,
    tags: [],
    isArchived: p.is_archived,
  };
}

export function applyProjectView(base: ApiProject, v: Project): ApiProject {
  return {
    ...base,
    title: v.title,
    description: v.description,
    color: v.color,
    is_archived: v.isArchived,
  };
}