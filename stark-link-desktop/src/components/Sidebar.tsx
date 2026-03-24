import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Monitor,
  ArrowUpDown,
  ClipboardList,
  Gamepad2,
  Settings,
  Zap,
} from "lucide-react";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/devices", icon: Monitor, label: "Devices" },
  { to: "/remote", icon: Gamepad2, label: "Remote" },
  { to: "/transfers", icon: ArrowUpDown, label: "Transfers" },
  { to: "/clipboard", icon: ClipboardList, label: "Clipboard" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

function Sidebar() {
  return (
    <aside className="w-56 h-full bg-dark-surface border-r border-dark-border flex flex-col shrink-0">
      {/* Logo */}
      <div className="flex items-center gap-2 px-5 py-5 border-b border-dark-border">
        <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-accent-gradient-start to-accent-gradient-end flex items-center justify-center">
          <Zap size={18} className="text-white" />
        </div>
        <span className="text-lg font-bold text-dark-text tracking-tight">
          Stark-Link
        </span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 py-4 px-3 flex flex-col gap-1">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-150 ${
                isActive
                  ? "bg-gradient-to-r from-accent-blue/20 to-accent-purple/10 text-white border border-accent-blue/30"
                  : "text-dark-text-secondary hover:text-dark-text hover:bg-dark-hover"
              }`
            }
          >
            <Icon size={18} />
            {label}
          </NavLink>
        ))}
      </nav>

      {/* Footer */}
      <div className="px-5 py-4 border-t border-dark-border">
        <p className="text-xs text-dark-text-secondary">Stark-Link v0.1.0</p>
      </div>
    </aside>
  );
}

export default Sidebar;
