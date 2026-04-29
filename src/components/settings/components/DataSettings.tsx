import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

interface DataSettingsProps {
  logRetentionDays: number;
  onLogRetentionDaysChange: (days: number) => void;
  onExportLogs: (format: "json" | "csv") => void;
}

export function DataSettings({
  logRetentionDays,
  onLogRetentionDaysChange,
  onExportLogs,
}: DataSettingsProps) {
  return (
    <section>
      <div className="border-b border-slate-200 py-3">
        <p className="text-sm font-bold text-slate-950">数据设置</p>
      </div>

      <div className="divide-y divide-slate-200">
        <div className="grid gap-3 py-4 md:grid-cols-[220px_minmax(0,1fr)_180px] md:items-center">
          <div className="text-sm font-semibold text-slate-950">日志保留天数</div>
          <p className="text-xs text-slate-500">自动删除此天数之前的日志</p>
          <Input
            type="number"
            min={1}
            max={3650}
            value={logRetentionDays}
            onChange={(e) => onLogRetentionDaysChange(Number(e.target.value) || 90)}
            className="h-9 rounded-none font-mono text-sm md:w-[180px]"
          />
        </div>

        <div className="grid gap-3 py-4 md:grid-cols-[220px_minmax(0,1fr)_180px] md:items-center">
          <div className="text-sm font-semibold text-slate-950">导出日志</div>
          <p className="text-xs text-slate-500">下载当前日志数据</p>
          <div className="grid grid-cols-2 gap-2 md:w-[180px]">
            <Button
              onClick={() => onExportLogs("json")}
              variant="outline"
              className="rounded-none font-mono"
            >
              JSON
            </Button>
            <Button
              onClick={() => onExportLogs("csv")}
              variant="outline"
              className="rounded-none font-mono"
            >
              CSV
            </Button>
          </div>
        </div>
      </div>
    </section>
  );
}
