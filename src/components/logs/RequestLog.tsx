import { useEffect, useState } from "react";
import {
  ListRestart,
  Download,
  Search,
  RotateCcw,
  Loader2,
  RefreshCw,
  Terminal,
  Radio,
  ScrollText,
} from "lucide-react";
import { toast } from "sonner";
import { getLogs, exportLogs, listProviders } from "@/lib/tauri";
import type { RequestLog as RequestLogType, Provider } from "@/lib/types";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/shared/EmptyState";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { DatePicker } from "@/components/ui/date-picker";

function formatTime(ts: string) {
  try {
    const d = new Date(ts);
    return d.toLocaleString("zh-CN", {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return ts;
  }
}

function statusVariant(code: number | null): "default" | "destructive" | "secondary" | "outline" {
  if (code === null) return "outline";
  if (code >= 200 && code < 300) return "default";
  if (code >= 300 && code < 400) return "secondary";
  return "destructive";
}

function formatLatency(ms: number | null) {
  if (ms === null) return "-";
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function SkeletonRow() {
  return (
    <TableRow>
      {Array.from({ length: 8 }).map((_, i) => (
        <TableCell key={i}>
          <Skeleton className="h-4 w-[60px]" />
        </TableCell>
      ))}
    </TableRow>
  );
}

export function RequestLog() {
  const [logs, setLogs] = useState<RequestLogType[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize] = useState(50);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [exporting, setExporting] = useState(false);

  // Draft filters
  const [draftProvider, setDraftProvider] = useState("");
  const [draftModel, setDraftModel] = useState("");
  const [draftStatus, setDraftStatus] = useState("");
  const [draftDateFrom, setDraftDateFrom] = useState("");
  const [draftDateTo, setDraftDateTo] = useState("");

  // Committed fetch params
  const [fetchKey, setFetchKey] = useState(0);

  // Detail dialog
  const [selectedLog, setSelectedLog] = useState<RequestLogType | null>(null);

  useEffect(() => {
    listProviders().then(setProviders).catch(console.error);
  }, []);

  const fetchLogs = async (p: number) => {
    setLoading(true);
    setError(null);
    try {
      const params: Record<string, unknown> = { page: p, page_size: pageSize };
      if (draftProvider) params.provider_id = draftProvider;
      if (draftModel) params.model = draftModel;
      if (draftStatus) params.status_code = parseInt(draftStatus);
      if (draftDateFrom) params.date_from = draftDateFrom;
      if (draftDateTo) params.date_to = draftDateTo;
      const result = await getLogs(params as any);
      setLogs(result.logs);
      setTotal(result.total);
      setPage(result.page);
    } catch (err) {
      setError(String(err));
      setLogs([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchLogs(1);
  }, [fetchKey]);

  const handleFilter = () => {
    setPage(1);
    fetchLogs(1);
  };

  const handleReset = () => {
    setDraftProvider("");
    setDraftModel("");
    setDraftStatus("");
    setDraftDateFrom("");
    setDraftDateTo("");
    setTimeout(() => setFetchKey((k) => k + 1), 0);
  };

  const handleRefresh = () => {
    fetchLogs(page);
  };

  const handleExport = async (format: "json" | "csv") => {
    setExporting(true);
    try {
      const params: Record<string, unknown> = { format };
      if (draftProvider) params.provider_id = draftProvider;
      if (draftModel) params.model = draftModel;
      if (draftStatus) params.status_code = parseInt(draftStatus);
      if (draftDateFrom) params.date_from = draftDateFrom;
      if (draftDateTo) params.date_to = draftDateTo;
      const result = await exportLogs(params as any);
      const blob = new Blob([result.content], { type: result.mime_type });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = result.filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      toast.success(`日志已导出为 ${format.toUpperCase()}`);
    } catch (err) {
      toast.error(`导出失败: ${err}`);
    } finally {
      setExporting(false);
    }
  };

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  if (error && logs.length === 0) {
    return (
      <div className="fade-in">
        <EmptyState
          icon={<ListRestart className="h-10 w-10" />}
          title="加载失败"
          description={error}
          action={<Button onClick={() => fetchLogs(page)}>重试</Button>}
        />
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 fade-in">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-foreground">请求日志</h1>
          <p className="text-sm text-muted-foreground mt-1">
            查看和导出所有代理请求记录
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={handleRefresh} disabled={loading}>
            <RefreshCw className={loading ? "animate-spin" : ""} />
            刷新
          </Button>
          <DropdownMenu>
            <DropdownMenuTrigger render={<Button variant="default" size="sm" disabled={exporting} />}>
              {exporting ? <Loader2 className="animate-spin" /> : <Download />}
              导出
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onMouseDown={() => handleExport("json")}>
                <ScrollText /> 导出 JSON
              </DropdownMenuItem>
              <DropdownMenuItem onMouseDown={() => handleExport("csv")}>
                <Terminal /> 导出 CSV
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-end gap-3">
        <div className="flex flex-col gap-1.5">
          <label className="text-xs font-medium text-muted-foreground">提供商</label>
          <Select value={draftProvider} onValueChange={(v) => setDraftProvider(v ?? "")}>
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="全部" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">全部</SelectItem>
              {providers.map((p) => (
                <SelectItem key={p.id} value={p.id}>
                  {p.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="flex flex-col gap-1.5">
          <label className="text-xs font-medium text-muted-foreground">模型</label>
          <Input
            placeholder="搜索模型..."
            className="h-9 w-[160px]"
            value={draftModel}
            onChange={(e) => setDraftModel(e.target.value)}
          />
        </div>

        <div className="flex flex-col gap-1.5">
          <label className="text-xs font-medium text-muted-foreground">状态码</label>

          <Select
            value={draftStatus}
            onValueChange={(value) => setDraftStatus(value ?? "")}
          >
            <SelectTrigger className="w-[110px]">
              <SelectValue placeholder="全部" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">全部</SelectItem>
              <SelectItem value="200">200 成功</SelectItem>
              <SelectItem value="400">400 错误</SelectItem>
              <SelectItem value="500">500 服务端错误</SelectItem>
              <SelectItem value="502">502 上游不可达</SelectItem>
              <SelectItem value="504">504 超时</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="flex flex-col gap-1.5">
          <label className="text-xs font-medium text-muted-foreground">起始日期</label>
          <DatePicker value={draftDateFrom} onChange={setDraftDateFrom} placeholder="起始日期" />
        </div>

        <div className="flex flex-col gap-1.5">
          <label className="text-xs font-medium text-muted-foreground">结束日期</label>
          <DatePicker value={draftDateTo} onChange={setDraftDateTo} placeholder="结束日期" />
        </div>

        <div className="flex items-center gap-2 pb-px">
          <Button variant="default" size="sm" onClick={handleFilter}>
            <Search /> 查询
          </Button>
          <Button variant="ghost" size="sm" onClick={handleReset}>
            <RotateCcw /> 重置
          </Button>
        </div>
      </div>

      {/* Count */}
      <div className="text-sm text-muted-foreground">
        共 <span className="font-medium text-foreground">{total}</span> 条记录
        {loading && <Loader2 className="ml-2 inline h-3.5 w-3.5 animate-spin" />}
      </div>

      {/* Table */}
      {logs.length === 0 && !loading ? (
        <EmptyState
          icon={<Radio className="h-10 w-10" />}
          title="暂无日志"
          description="当有代理请求通过时，日志将显示在这里"
        />
      ) : (
        <div className="rounded-lg border border-border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-[130px]">时间</TableHead>
                <TableHead className="w-[140px]">模型</TableHead>
                <TableHead className="w-[100px]">提供商</TableHead>
                <TableHead className="w-[60px]">状态</TableHead>
                <TableHead className="w-[70px]">延迟</TableHead>
                <TableHead className="w-[70px] text-right">输入</TableHead>
                <TableHead className="w-[70px] text-right">输出</TableHead>
                <TableHead className="w-[50px]">流式</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {loading
                ? Array.from({ length: 8 }).map((_, i) => <SkeletonRow key={i} />)
                : logs.map((log) => (
                    <TableRow
                      key={log.id}
                      className="cursor-pointer"
                      onClick={() => setSelectedLog(log)}
                    >
                      <TableCell className="text-muted-foreground text-xs">
                        {formatTime(log.timestamp)}
                      </TableCell>
                      <TableCell className="max-w-[140px] truncate font-medium">
                        {log.model}
                      </TableCell>
                      <TableCell>{log.provider_name}</TableCell>
                      <TableCell>
                        {log.status_code !== null && (
                          <Badge variant={statusVariant(log.status_code)}>
                            {log.status_code}
                          </Badge>
                        )}
                      </TableCell>
                      <TableCell className="text-muted-foreground tabular-nums">
                        {formatLatency(log.latency_ms)}
                      </TableCell>
                      <TableCell className="text-right tabular-nums text-muted-foreground">
                        {log.input_tokens.toLocaleString()}
                      </TableCell>
                      <TableCell className="text-right tabular-nums text-muted-foreground">
                        {log.output_tokens.toLocaleString()}
                      </TableCell>
                      <TableCell>
                        {log.is_streaming ? (
                          <Badge variant="outline" className="text-[10px]">
                            流式
                          </Badge>
                        ) : (
                          <span className="text-muted-foreground text-xs">-</span>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
            </TableBody>
          </Table>
        </div>
      )}

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-between">
          <div className="text-xs text-muted-foreground">
            第 {page}/{totalPages} 页
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={page <= 1 || loading}
              onClick={() => fetchLogs(page - 1)}
            >
              上一页
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={page >= totalPages || loading}
              onClick={() => fetchLogs(page + 1)}
            >
              下一页
            </Button>
          </div>
        </div>
      )}

      {/* Detail Dialog */}
      <Dialog open={!!selectedLog} onOpenChange={(open) => !open && setSelectedLog(null)}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>请求详情</DialogTitle>
          </DialogHeader>
          {selectedLog && (
            <div className="space-y-3 text-sm">
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">请求 ID</span>
                <span className="col-span-2 font-mono text-xs break-all">
                  {selectedLog.request_id}
                </span>
              </div>
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">时间</span>
                <span className="col-span-2">{selectedLog.timestamp}</span>
              </div>
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">模型</span>
                <span className="col-span-2">{selectedLog.model}</span>
              </div>
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">提供商</span>
                <span className="col-span-2">{selectedLog.provider_name}</span>
              </div>
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">协议</span>
                <span className="col-span-2">{selectedLog.protocol}</span>
              </div>
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">上游地址</span>
                <span className="col-span-2 font-mono text-xs break-all">
                  {selectedLog.upstream_url}
                </span>
              </div>
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">状态码</span>
                <span className="col-span-2">
                  {selectedLog.status_code !== null ? (
                    <Badge variant={statusVariant(selectedLog.status_code)}>
                      {selectedLog.status_code}
                    </Badge>
                  ) : (
                    "-"
                  )}
                </span>
              </div>
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">延迟</span>
                <span className="col-span-2">{formatLatency(selectedLog.latency_ms)}</span>
              </div>
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">输入 Token</span>
                <span className="col-span-2 tabular-nums">
                  {selectedLog.input_tokens.toLocaleString()}
                </span>
              </div>
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">输出 Token</span>
                <span className="col-span-2 tabular-nums">
                  {selectedLog.output_tokens.toLocaleString()}
                </span>
              </div>
              <div className="grid grid-cols-3 gap-1">
                <span className="text-muted-foreground">流式</span>
                <span className="col-span-2">
                  {selectedLog.is_streaming ? "是" : "否"}
                </span>
              </div>
              {selectedLog.error_message && (
                <div className="grid grid-cols-3 gap-1">
                  <span className="text-muted-foreground">错误信息</span>
                  <span className="col-span-2 text-destructive break-all">
                    {selectedLog.error_message}
                  </span>
                </div>
              )}
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
