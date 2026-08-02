import { useState } from "react";
import { NavLink as RouterNavLink, useLocation } from "react-router-dom";
import {
  AppShell,
  Avatar,
  Code,
  Divider,
  Group,
  NavLink,
  ScrollArea,
  TextInput,
} from "@mantine/core";
import { IconChevronRight } from "@tabler/icons-react";
import iconUrl from "../../assets/icon.svg";

{ /* pages that will show up in navbar */ }
const pages = [
  { label: "Feed", to: "/feed" },
  { label: "Notes", to: "/notes" },
  { label: "Tasks", to: "/tasks" },
  { label: "Calendar", to: "/calendar" },
  { label: "Knowledge Graph", to: "/graph" },
  { label: "Tags", to: "/tags" },
];

export function AppNavbar() {
  const { pathname } = useLocation();
  const [search, setSearch] = useState("");

  return (
    <>
      <AppShell.Section>
        { /* icon and version */}
        <Group justify="space-between" mb="md">
          <img src={iconUrl} alt="Explorer" height={32} />
          { /* get current version from vite */}
          <Code fw={700}>v{import.meta.env.VITE_APP_VERSION}</Code>
        </Group>

        <Divider mb="md" mx="-md" />

        { /* search bar */}
        <TextInput
          placeholder="Search"
          size="xs"
          value={search}
          onChange={(e) => setSearch(e.currentTarget.value)}
          aria-label="Search"
        />
      </AppShell.Section>

      <AppShell.Section grow my="md" component={ScrollArea}>
        { /* pages in navbar */}
        {pages.map((page) => (
          <NavLink
            key={page.to}
            component={RouterNavLink}
            to={page.to}
            label={page.label}
            active={pathname === page.to}
          />
        ))}
      </AppShell.Section>

      <Divider mx="-md" />

      <AppShell.Section pt="md">
        { /* user/settings */}
        <NavLink
          component={RouterNavLink}
          to="/settings"
          active={pathname === "/settings"}
          label="User" // TODO: replace placeholder user
          leftSection={
            <Avatar radius="xl" size="sm" color="indigo">
              U
            </Avatar>
          }
          rightSection={<IconChevronRight size={16} stroke={1.5} />}
        />
      </AppShell.Section>
    </>
  );
}