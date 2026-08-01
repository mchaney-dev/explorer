import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Title } from "@mantine/core";
import { AppLayout } from "./components/shared/AppShell";
import { TasksPage } from "./pages/TasksPage";

// TODO: replace placeholders
function Placeholder({ name }: { name: string }) {
  return <Title order={2}>{name}</Title>;
}

function App() {
  return (
    <BrowserRouter>
      <AppLayout>
        <Routes>
          {/* TODO: replace with default home from user preferences */}
          <Route path="/" element={<Navigate to="/feed" replace />} />
          <Route path="/feed" element={<Placeholder name="Feed" />} />
          <Route path="/notes" element={<Placeholder name="Notes" />} />
          <Route path="/tasks" element={<TasksPage />} />
          <Route path="/calendar" element={<Placeholder name="Calendar" />} />
          <Route path="/graph" element={<Placeholder name="Knowledge Graph" />} />
          <Route path="/tags" element={<Placeholder name="Tags" />} />
          <Route path="/settings" element={<Placeholder name="Settings" />} />
        </Routes>
      </AppLayout>
    </BrowserRouter>
  );
}

export default App;
