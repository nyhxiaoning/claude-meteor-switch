import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";

interface ProxySettingsProps {
  proxyPort: number;
  autoStartProxy: boolean;
  onProxyPortChange: (port: number) => void;
  onAutoStartProxyChange: (enabled: boolean) => void;
}

export function ProxySettings({
  proxyPort,
  autoStartProxy,
  onProxyPortChange,
  onAutoStartProxyChange,
}: ProxySettingsProps) {
  return (
    <section>
      <div className="border-b border-slate-200 py-3">
        <p className="text-sm font-bold text-slate-950">代理设置</p>
      </div>

      <div className="divide-y divide-slate-200">
        <div className="grid gap-3 py-4 md:grid-cols-[220px_minmax(0,1fr)_180px] md:items-center">
          <div className="text-sm font-semibold text-slate-950">代理端口</div>
          <p className="text-xs text-slate-500">代理服务器运行端口</p>
          <Input
            type="number"
            min={1}
            max={65535}
            value={proxyPort}
            onChange={(e) => onProxyPortChange(Number(e.target.value) || 9876)}
            className="h-9 rounded-none font-mono text-sm md:w-[180px]"
          />
        </div>

        <div className="grid gap-3 py-4 md:grid-cols-[220px_minmax(0,1fr)_180px] md:items-center">
          <div className="text-sm font-semibold text-slate-950">启动时运行代理</div>
          <p className="text-xs text-slate-500">应用启动时自动启动代理服务器</p>
          <div className="flex md:justify-end">
            <Switch
              checked={autoStartProxy}
              onChange={(e) => onAutoStartProxyChange(e.target.checked)}
            />
          </div>
        </div>
      </div>
    </section>
  );
}
