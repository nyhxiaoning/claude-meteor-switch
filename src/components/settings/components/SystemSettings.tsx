import { Switch } from "@/components/ui/switch";

interface SystemSettingsProps {
  autostartEnabled: boolean;
  autostartLoading: boolean;
  onAutostartChange: (enabled: boolean) => void;
}

export function SystemSettings({
  autostartEnabled,
  autostartLoading,
  onAutostartChange,
}: SystemSettingsProps) {
  return (
    <section>
      <div className="border-b border-slate-200 py-3">
        <p className="text-sm font-bold text-slate-950">系统设置</p>
      </div>

      <div className="grid gap-3 border-b border-slate-200 py-4 md:grid-cols-[220px_minmax(0,1fr)_180px] md:items-center">
        <div className="text-sm font-semibold text-slate-950">开机启动应用</div>
        <p className="text-xs text-slate-500">系统启动时自动启动应用</p>
        <div className="flex md:justify-end">
          <Switch
            checked={autostartEnabled}
            onChange={(e) => onAutostartChange(e.target.checked)}
            disabled={autostartLoading}
          />
        </div>
      </div>
    </section>
  );
}
