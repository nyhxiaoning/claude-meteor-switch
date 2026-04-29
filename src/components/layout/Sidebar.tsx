import { NavLink } from "react-router-dom";
import { useEffect, useState } from "react";
import { LayoutDashboard, Link2, Network, ScrollText, Settings } from "lucide-react";
import { getProxyStatus } from "@/lib/tauri";
import type { ProxyStatus } from "@/lib/types";

const navItems = [
  { to: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { to: "/providers", label: "Providers", icon: Network },
  { to: "/logs", label: "Logs", icon: ScrollText },
  { to: "/integration", label: "Integration", icon: Link2 },
];

export function Sidebar() {
  const [status, setStatus] = useState<ProxyStatus | null>(null);

  useEffect(() => {
    const poll = () => {
      getProxyStatus()
        .then(setStatus)
        .catch(() => setStatus(null));
    };
    poll();
    const id = setInterval(poll, 5000);
    return () => clearInterval(id);
  }, []);

  return (
    <>
      <aside className="hidden h-screen w-56 shrink-0 flex-col border-r border-slate-800 bg-slate-900/80 backdrop-blur-xl lg:flex">
        {/* Logo */}
        <div className="flex items-center gap-3 border-b border-slate-800 px-5 py-5">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-cyan-500/10 text-cyan-400 font-mono text-lg font-bold glow-border float-animation">
            M
          </div>
          <div className="flex flex-col">
            <span className="text-slate-100 font-semibold tracking-tight">METEOR</span>
            <span className="text-slate-500 text-xs">v1.0</span>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex flex-1 flex-col gap-2 px-4 py-6">
          {navItems.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                `group flex items-center gap-3 rounded-xl px-4 py-3 transition-all duration-300 ${
                  isActive
                    ? "bg-cyan-500/15 text-cyan-400 border border-cyan-500/30 shadow-[0_0_20px_rgba(0,255,255,0.1)]"
                    : "text-slate-400 hover:bg-slate-800/50 hover:text-slate-200"
                }`
              }
            >
              <Icon className="h-5 w-5" />
              <span className="text-sm font-medium">{label}</span>
            </NavLink>
          ))}
        </nav>

        {/* Divider */}
        <div className="sci-fi-divider mx-4" />

        {/* Settings & Status Panel */}
        <div className="px-4 py-5">
          <div className="sci-fi-card rounded-xl p-4">
            <div className="flex items-center gap-3">
              {/* Settings Icon */}
              <NavLink
                key="/settings"
                to="/settings"
                className={({ isActive }) =>
                  `flex h-10 w-10 items-center justify-center rounded-lg border transition-all ${
                    isActive
                      ? "border-cyan-500/40 bg-cyan-500/15 text-cyan-400 shadow-[0_0_15px_rgba(0,255,255,0.15)]"
                      : "border-slate-700 text-slate-400 hover:bg-slate-800 hover:text-slate-200 hover:border-slate-600"
                  }`
                }
              >
                <Settings className="h-5 w-5" />
              </NavLink>

              {/* Proxy Status */}
              <div className="flex-1">
                <div className="flex items-center gap-2">
                  <div
                    className={`h-3 w-3 rounded-full ${
                      status?.running ? "bg-cyan-400 status-dot" : "bg-slate-600"
                    }`}
                  />
                  <span className="text-slate-300 font-mono text-xs font-medium">
                    {status?.running ? "ONLINE" : "OFFLINE"}
                  </span>
                </div>
                {status?.port && (
                  <div className="mt-1">
                    <span className="text-cyan-400/80 font-mono text-xs">{status.port}</span>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </aside>

      {/* Mobile Navigation */}
      <nav className="fixed inset-x-0 bottom-0 z-20 flex items-center justify-between border-t border-slate-800 bg-slate-900/95 backdrop-blur-xl px-2 py-2 lg:hidden">
        {[...navItems, { to: "/settings", label: "Settings", icon: Settings }].map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              `flex min-w-0 flex-1 flex-col items-center gap-1 rounded-lg px-2 py-2 text-center transition-all ${
                isActive ? "text-cyan-400 bg-cyan-500/10" : "text-slate-500"
              }`
            }
          >
            <item.icon className="h-5 w-5" />
            <span className="truncate text-[10px] font-medium">{item.label}</span>
          </NavLink>
        ))}
      </nav>
    </>
  );
}
