import {
  Chip,
  Group,
  NumberInput,
  Select,
  Stack,
  Text,
} from "@mantine/core";
import { DateInput } from "@mantine/dates";
import { WEEKDAYS, type RecurrenceDraft } from "./types";

const frequencyOptions = [
  { value: "daily", label: "Daily" },
  { value: "weekly", label: "Weekly" },
  { value: "monthly", label: "Monthly" },
  { value: "yearly", label: "Yearly" },
];

const intervalUnit: Record<string, string> = {
  daily: "day(s)",
  weekly: "week(s)",
  monthly: "month(s)",
  yearly: "year(s)",
};

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

interface RecurrenceEditorProps {
  value: RecurrenceDraft | null;
  onChange: (value: RecurrenceDraft | null) => void;
}

export function RecurrenceEditor({ value, onChange }: RecurrenceEditorProps) {
  function handleFrequency(frequency: string | null) {
    if (!frequency) {
      onChange(null);
      return;
    }
    if (!value) {
      onChange({
        frequency,
        interval: 1,
        daysOfWeek: [],
        dayOfMonth: null,
        endDate: null,
      });
      return;
    }
    onChange({ ...value, frequency });
  }

  function patch(fields: Partial<RecurrenceDraft>) {
    if (!value) return;
    onChange({ ...value, ...fields });
  }

  return (
    <Stack gap="xs">
      <Select
        label="Recurrence"
        placeholder="Does not repeat"
        data={frequencyOptions}
        value={value?.frequency ?? null}
        onChange={handleFrequency}
        clearable
      />

      {value && (
        <Stack gap="xs" pl="xs">
          <NumberInput
            label={`Repeat every`}
            suffix={` ${intervalUnit[value.frequency] ?? ""}`}
            min={1}
            value={value.interval ?? 1}
            onChange={(v) =>
              patch({ interval: typeof v === "number" ? v : 1 })
            }
            maw={220}
          />

          {value.frequency === "weekly" && (
            <div>
              <Text size="sm" fw={500} mb={4}>
                On days
              </Text>
              <Chip.Group
                multiple
                value={value.daysOfWeek}
                onChange={(days) => patch({ daysOfWeek: days })}
              >
                <Group gap="xs">
                  {WEEKDAYS.map((day) => (
                    <Chip key={day.value} value={day.value} size="sm">
                      {day.label}
                    </Chip>
                  ))}
                </Group>
              </Chip.Group>
            </div>
          )}

          {value.frequency === "monthly" && (
            <NumberInput
              label="Day of month"
              placeholder="1–31"
              min={1}
              max={31}
              value={value.dayOfMonth ?? ""}
              onChange={(v) =>
                patch({ dayOfMonth: typeof v === "number" ? v : null })
              }
              maw={220}
            />
          )}

          <DateInput
            label="Ends on"
            placeholder="Never"
            value={toDateString(value.endDate)}
            onChange={(v) => patch({ endDate: toDate(v) })}
            clearable
            maw={220}
          />
        </Stack>
      )}
    </Stack>
  );
}