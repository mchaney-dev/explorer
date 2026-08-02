import { useEffect, useState } from "react";
import {
  Button,
  ColorInput,
  Group,
  Modal,
  Stack,
  TagsInput,
  Textarea,
  TextInput,
} from "@mantine/core";
import type { Project } from "./types";

interface NewProjectModalProps {
  opened: boolean;
  onClose: () => void;
  initial: Project | null;
  onSubmit: (project: Project) => void;
}

export function NewProjectModal({
  opened,
  onClose,
  initial,
  onSubmit,
}: NewProjectModalProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [color, setColor] = useState("");
  const [tags, setTags] = useState<string[]>([]);

  useEffect(() => {
    if (!opened) return;
    if (initial) {
      setTitle(initial.title);
      setDescription(initial.description);
      setColor(initial.color ?? "");
      setTags(initial.tags);
    } else {
      setTitle("");
      setDescription("");
      setColor("");
      setTags([]);
    }
  }, [opened, initial]);

  function handleSubmit() {
    onSubmit({
      id: initial?.id ?? crypto.randomUUID(),
      title,
      description,
      color: color || null,
      tags,
      isArchived: initial?.isArchived ?? false,
    });
    onClose();
  }

  const editing = initial !== null;

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={editing ? "Edit Project" : "New Project"}
      centered
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

        <ColorInput
          label="Color"
          placeholder="Pick a color"
          value={color}
          onChange={setColor}
          maw={220}
        />

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
