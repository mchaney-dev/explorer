import type {
  DayOfWeek,
  Frequency,
  Priority,
  Project as ApiProject,
  RecurrenceRule as ApiRule,
  Status,
  Tag,
  Task as ApiTask,
} from "../../bindings";
import {
  UNASSIGNED,
  type Project,
  type RecurrenceDraft,
  type Task,
} from "./types";

export function draftFromRule(rule: ApiRule): RecurrenceDraft {
  return {
    frequency: rule.frequency ?? "daily",
    interval: rule.interval,
    daysOfWeek: rule.days_of_week,
    dayOfMonth: rule.day_of_month,
    endDate: rule.end_date ? new Date(rule.end_date) : null,
  };
}

export function ruleFromDraft(base: ApiRule, draft: RecurrenceDraft): ApiRule {
  return {
    ...base,
    frequency: draft.frequency as Frequency,
    interval: draft.interval,
    days_of_week:
      draft.frequency === "weekly" ? (draft.daysOfWeek as DayOfWeek[]) : [],
    day_of_month: draft.frequency === "monthly" ? draft.dayOfMonth : null,
    end_date: draft.endDate ? draft.endDate.toISOString() : null,
  };
}

export function taskFromApi(t: ApiTask, rulesById: Map<string, ApiRule>): Task {
  const rule = t.recurrence_id ? rulesById.get(t.recurrence_id) : undefined;
  return {
    id: t.id,
    title: t.title,
    description: t.description,
    projectId: t.project_id ?? UNASSIGNED,
    status: t.status ?? "todo",
    priority: t.priority ?? "medium",
    dueDate: t.due_date ? new Date(t.due_date) : null,
    parentId: t.parent_id,
    recurrence: rule ? draftFromRule(rule) : null,
    tags: t.tags.map((tag) => tag.label),
  };
}

export function applyTaskView(base: ApiTask, v: Task, tags: Tag[]): ApiTask {
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
    tags,
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
    tags: p.tags.map((tag) => tag.label),
    isArchived: p.is_archived,
  };
}

export function applyProjectView(
  base: ApiProject,
  v: Project,
  tags: Tag[],
): ApiProject {
  return {
    ...base,
    title: v.title,
    description: v.description,
    color: v.color,
    is_archived: v.isArchived,
    tags,
  };
}