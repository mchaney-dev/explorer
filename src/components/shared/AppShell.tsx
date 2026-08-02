import React from "react";
import { AppShell } from "@mantine/core";
import { AppNavbar } from "./AppNavbar";

export function AppLayout({ children }: { children?: React.ReactNode }) {
  return (
    <AppShell
      navbar={{ width: 220, breakpoint: "sm", collapsed: { mobile: false } }}
      padding="md"
    >
      <AppShell.Navbar p="md">
        <AppNavbar />
      </AppShell.Navbar>

      <AppShell.Main>{children}</AppShell.Main>
    </AppShell>
  );
}
