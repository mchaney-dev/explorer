export const UNASSIGNED = "unassigned";

export interface Task {
  id: string;
  title: string;
  description: string;
  projectId: string;
  status: string;
  priority: string;
  dueDate: Date | null;
  parentId: string | null;
  recurrence: string | null;
  tags: string[];
}

export interface Project {
  id: string;
  title: string;
  description: string;
  color: string | null;
  tags: string[];
  isArchived: boolean;
}