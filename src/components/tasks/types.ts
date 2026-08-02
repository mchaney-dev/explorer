export const UNASSIGNED = "unassigned";

export const WEEKDAYS = [
  { value: "monday", label: "Mon" },
  { value: "tuesday", label: "Tue" },
  { value: "wednesday", label: "Wed" },
  { value: "thursday", label: "Thu" },
  { value: "friday", label: "Fri" },
  { value: "saturday", label: "Sat" },
  { value: "sunday", label: "Sun" },
] as const;

export interface RecurrenceDraft {
  frequency: string;
  interval: number | null;
  daysOfWeek: string[];
  dayOfMonth: number | null;
  endDate: Date | null;
}

export interface Task {
  id: string;
  title: string;
  description: string;
  projectId: string;
  status: string;
  priority: string;
  dueDate: Date | null;
  parentId: string | null;
  recurrence: RecurrenceDraft | null;
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