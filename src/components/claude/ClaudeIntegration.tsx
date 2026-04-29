
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { checkClaudeConfig, getProxyStatus, injectClaudeConfig, revertClaudeConfig } from "@/lib/tauri";
import type { ClaudeConfigStatus, ProxyStatus } from "@/lib/types";
import { ClaudeHeader } from "./components/ClaudeHeader";
import { ClaudeStatusCard } from "./components/ClaudeStatusCard";
import { Button } from "@/components/ui/button";

export function ClaudeIntegration() {
  const [config, setConfig] = useState<ClaudeConfigStatus | null>(null);
  const [proxyStatus, setProxyStatus] = useState<ProxyStatus | null>(null);
  const [isInjecting, setIsInjecting] = useState(false);
  const [isReverting, setIsReverting] = useState(false);

  const refresh = async () => {
    try {
      const [nextConfig, nextStatus] = await Promise.all([checkClaudeConfig(), getProxyStatus()]);
      setConfig(nextConfig);
      setProxyStatus(nextStatus);
    } catch {}
  };

  useEffect(() => {
    refresh();
  }, []);

  const handleInject = async () => {
    setIsInjecting(true);
    try {
      await injectClaudeConfig();
      toast.success("Claude Code 已配置");
      await refresh();
    } catch (error) {
      toast.error(`配置失败: ${error}`);
    } finally {
      setIsInjecting(false);
    }
  };

  const handleRevert = async () => {
    setIsReverting(true);
    try {
      await revertClaudeConfig();
      toast.success("Claude Code 已还原");
      await refresh();
    } catch (error) {
      toast.error(`还原失败: ${error}`);
    } finally {
      setIsReverting(false);
    }
  };

  return (
    <div className="flex flex-col gap-6 fade-in">
      <ClaudeHeader />
      <ClaudeStatusCard config={config} proxyStatus={proxyStatus} />

      <div className="flex gap-4">
        <Button
          onClick={handleInject}
          disabled={isInjecting || isReverting}
          size="lg"
          className="flex-1"
        >
          {isInjecting ? "配置中..." : "配置 Claude Code"}
        </Button>
        {config?.is_configured && (
          <Button
            onClick={handleRevert}
            disabled={isInjecting || isReverting}
            variant="destructive"
            size="lg"
            className="flex-1"
          >
            {isReverting ? "还原中..." : "还原为默认"}
          </Button>
        )}
      </div>
    </div>
  );
}
