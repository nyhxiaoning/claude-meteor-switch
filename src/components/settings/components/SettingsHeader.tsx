import { Terminal } from "lucide-react";

export function SettingsHeader() {
  return (
    <div className="flex flex-col gap-3 border-b border-slate-200 pb-5 sm:flex-row sm:items-end sm:justify-between">
      <div>
        <p className="font-mono text-xs font-semibold uppercase tracking-wide text-slate-500">
          配置
        </p>
        <h1 className="mt-1 text-2xl font-semibold tracking-normal text-foreground">设置</h1>
      </div>
      <div className="flex items-center gap-2 font-mono text-xs font-semibold text-slate-500">
        <Terminal className="h-3.5 w-3.5" />
        本地设置
      </div>
    </div>
  );
}
