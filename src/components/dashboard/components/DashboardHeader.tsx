
import { Play, Square } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { ProxyStatus } from "@/lib/types";

interface DashboardHeaderProps {
  proxyStatus: ProxyStatus | null;
  onToggleProxy: () => void;
}

export function DashboardHeader({ proxyStatus, onToggleProxy }: DashboardHeaderProps) {
  return (
    <div className="flex items-center justify-between">
      <h1 className="text-2xl font-semibold">仪表盘</h1>
      <Button onClick={onToggleProxy} variant={proxyStatus?.running ? "destructive" : "default"}>
        {proxyStatus?.running ? (
          <>
            <Square className="mr-2 h-4 w-4" /> 停止代理
          </>
        ) : (
          <>
            <Play className="mr-2 h-4 w-4" /> 启动代理
          </>
        )}
      </Button>
    </div>
  );
}
