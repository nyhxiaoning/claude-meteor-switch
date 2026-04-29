
import { Link2 } from "lucide-react";
import type { ClaudeConfigStatus, ProxyStatus } from "@/lib/types";

interface ClaudeStatusCardProps {
  config: ClaudeConfigStatus | null;
  proxyStatus: ProxyStatus | null;
}

export function ClaudeStatusCard({ config, proxyStatus }: ClaudeStatusCardProps) {
  return (
    <div className="signal-panel p-6 hover-lift">
      <div className="flex items-start gap-4 mb-6">
        <div className="flex h-12 w-12 items-center justify-center bg-primary/10 text-primary rounded-md">
          <Link2 className="h-6 w-6" />
        </div>
        <div className="flex-1">
          <h2 className="text-lg font-semibold text-foreground">集成状态</h2>
          <p className="text-muted-foreground text-sm mt-1">
            配置 Claude Code 使用您的代理而不是默认 API 端点。
          </p>
        </div>
      </div>

      <div className="space-y-4">
        <div className="flex items-center gap-4">
          <div
            className={`h-3 w-3 rounded-full ${proxyStatus?.running ? "bg-primary status-dot" : "bg-muted-foreground"}`}
          />
          <span className="text-foreground text-sm font-medium">
            {proxyStatus?.running ? "代理: 运行中" : "代理: 已停止"}
          </span>
          {proxyStatus?.port && (
            <span className="text-muted-foreground text-sm font-mono bg-muted border border-border px-3 py-1.5 rounded-md">
              127.0.0.1:{proxyStatus.port}
            </span>
          )}
        </div>

        <div className="flex items-center gap-4">
          <div
            className={`h-3 w-3 rounded-full ${config?.is_configured ? "bg-primary status-dot" : "bg-destructive"}`}
          />
          <span className="text-foreground text-sm font-medium">
            {config?.is_configured ? "Claude Code: 已配置" : "Claude Code: 未配置"}
          </span>
        </div>

        {config && config.base_url && (
          <div className="text-muted-foreground text-sm font-mono bg-muted border border-border px-4 py-3 rounded-md">
            <div className="font-medium mb-1 text-foreground">ANTHROPIC_BASE_URL:</div>
            <div>{config.base_url}</div>
          </div>
        )}
      </div>
    </div>
  );
}
