import { useEffect, useState } from "react";
import {
  Button,
  Group,
  Modal,
  Select,
  Stack,
  TagsInput,
  Textarea,
  TextInput,
} from "@mantine/core";
import { DateInput } from "@mantine/dates";
import { RecurrenceEditor } from "./RecurrenceEditor";
import { UNASSIGNED, type RecurrenceDraft, type Task } from "./types";

function toDate(value: string | null): Date | null {
  return value ? new Date(`${value}T00:00:00`) : null;
}
function toDateString(date: Date | null): string | null {
  if (!date) return null;
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

const statusOptions = [
  { value: "todo", label: "To Do" },
  { value: "in_progress", label: "In Progress" },
  { value: "done", label: "Done" },
  { value: "cancelled", label: "Cancelled" },
];

const priorityOptions = [
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
  { value: "urgent", label: "Urgent" },
];

interface NewTaskModalProps {
  opened: boolean;
  onClose: () => void;
  projectOptions: { value: string; label: string }[];
  parentOptions: { value: string; label: string }[];
  defaultProjectId: string;
  initial: Task | null;
  onSubmit: (task: Task) => void;
}

export function NewTaskModal({
  opened,
  onClose,
  projectOptions,
  parentOptions,
  defaultProjectId,
  initial,
  onSubmit,
}: NewTaskModalProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [projectId, setProjectId] = useState<string | null>(defaultProjectId);
  const [status, setStatus] = useState<string | null>("todo");
  const [priority, setPriority] = useState<string | null>("medium");
  const [dueDate, setDueDate] = useState<Date | null>(null);
  const [parentId, setParentId] = useState<string | null>(null);
  const [recurrence, setRecurrence] = useState<RecurrenceDraft | null>(null);
  const [tags, setTags] = useState<string[]>([]);

  useEffect(() => {
    if (!opened) return;
    if (initial) {
      setTitle(initial.title);
      setDescription(initial.description);
      setProjectId(initial.projectId);
      setStatus(initial.status);
      setPriority(initial.priority);
      setDueDate(initial.dueDate);
      setParentId(initial.parentId);
      setRecurrence(initial.recurrence);
      setTags(initial.tags);
    } else {
      setTitle("");
      setDescription("");
      setProjectId(defaultProjectId);
      setStatus("todo");
      setPriority("medium");
      setDueDate(null);
      setParentId(null);
      setRecurrence(null);
      setTags([]);
    }
  }, [opened, initial, defaultProjectId]);

  function handleSubmit() {
    onSubmit({
      id: initial?.id ?? crypto.randomUUID(),
      title,
      description,
      projectId: projectId ?? UNASSIGNED,
      status: status ?? "todo",
      priority: priority ?? "medium",
      dueDate,
      parentId,
      recurrence,
      tags,
    });
    onClose();
  }

  const editing = initial !== null;

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={editing ? "Edit Task" : "New Task"}
      centered
      size="lg"
    >
      <Stack>
        <TextInput
          label="Title"
          value={title}
          onChange={(e) => setTitle(e.currentTarget.value)}
          data-autofocus
        />

        <Textarea
          label="Description"
          value={description}
          onChange={(e) => setDescription(e.currentTarget.value)}
          autosize
          minRows={2}
        />

        <Select
          label="Project"
          data={projectOptions}
          value={projectId}
          onChange={setProjectId}
          allowDeselect={false}
        />

        <Group grow>
          <Select
            label="Status"
            data={statusOptions}
            value={status}
            onChange={setStatus}
          />
          <Select
            label="Priority"
            data={priorityOptions}
            value={priority}
            onChange={setPriority}
          />
        </Group>

        <DateInput
          label="Due date"
          placeholder="Pick a date"
          value={toDateString(dueDate)}
          onChange={(value) => setDueDate(toDate(value))}
          clearable
          maw={200}
        />

        <Select
          label="Parent task"
          placeholder="None"
          data={parentOptions}
          value={parentId}
          onChange={setParentId}
          clearable
          searchable
        />

        <RecurrenceEditor value={recurrence} onChange={setRecurrence} />

        <TagsInput
          label="Tags"
          placeholder="Type and press Enter"
          value={tags}
          onChange={setTags}
        />

        <Group justify="flex-end" mt="sm">
          <Button variant="default" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSubmit}>{editing ? "Save" : "Create"}</Button>
        </Group>
      </Stack>
    </Modal>
  );
}
