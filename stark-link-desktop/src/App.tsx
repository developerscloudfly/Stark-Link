import { Routes, Route } from "react-router-dom";
import Sidebar from "./components/Sidebar";
import Dashboard from "./pages/Dashboard";
import Devices from "./pages/Devices";
import Transfers from "./pages/Transfers";
import Clipboard from "./pages/Clipboard";
import Settings from "./pages/Settings";

function App() {
  return (
    <div className="flex h-screen w-screen bg-dark-bg overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-y-auto p-6">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/devices" element={<Devices />} />
          <Route path="/transfers" element={<Transfers />} />
          <Route path="/clipboard" element={<Clipboard />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
