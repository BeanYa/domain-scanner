import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";

const App = () => {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Navigate to="/dashboard" replace />} />
        <Route path="/dashboard" element={<div className="p-8 text-cyber-muted">Dashboard - Coming Soon</div>} />
        <Route path="/tasks" element={<div className="p-8 text-cyber-muted">TaskList - Coming Soon</div>} />
        <Route path="/tasks/new" element={<div className="p-8 text-cyber-muted">NewTask - Coming Soon</div>} />
        <Route path="/tasks/:id" element={<div className="p-8 text-cyber-muted">TaskDetail - Coming Soon</div>} />
        <Route path="/filter" element={<div className="p-8 text-cyber-muted">FilterResults - Coming Soon</div>} />
        <Route path="/vectorize" element={<div className="p-8 text-cyber-muted">VectorizePage - Coming Soon</div>} />
        <Route path="/proxies" element={<div className="p-8 text-cyber-muted">ProxyManager - Coming Soon</div>} />
        <Route path="/settings" element={<div className="p-8 text-cyber-muted">Settings - Coming Soon</div>} />
      </Routes>
    </BrowserRouter>
  );
};

export default App;
