import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import AppLayout from "./components/Layout/AppLayout";
import Dashboard from "./pages/Dashboard";
import TaskList from "./pages/TaskList";
import NewTask from "./pages/NewTask";
import TaskDetail from "./pages/TaskDetail";
import FilterResults from "./pages/FilterResults";
import VectorizePage from "./pages/VectorizePage";
import ProxyManager from "./pages/ProxyManager";
import Settings from "./pages/Settings";
import TaskStatusNotifications from "./components/TaskStatusNotifications";

const App = () => {
  return (
    <BrowserRouter future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
      <Routes>
        <Route path="/" element={<AppLayout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<Dashboard />} />
          <Route path="tasks" element={<TaskList />} />
          <Route path="tasks/new" element={<NewTask />} />
          <Route path="tasks/:id" element={<TaskDetail />} />
          <Route path="filter" element={<FilterResults />} />
          <Route path="vectorize" element={<VectorizePage />} />
          <Route path="proxies" element={<ProxyManager />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
      <TaskStatusNotifications />
    </BrowserRouter>
  );
};

export default App;
