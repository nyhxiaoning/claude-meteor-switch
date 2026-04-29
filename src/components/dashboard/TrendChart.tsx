import {
  CartesianGrid,
  Bar,
  BarChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

interface TrendData {
  date: string;
  total_requests: number;
  total_errors: number;
  avg_latency_ms: number;
}

interface TrendChartProps {
  data: TrendData[];
}

export function TrendChart({ data }: TrendChartProps) {
  if (data.length === 0) {
    return (
      <div className="signal-panel flex h-56 items-center justify-center">
        <p className="text-sm text-muted-foreground">暂无趋势数据</p>
      </div>
    );
  }

  const chartData = data.map((item) => ({
    date: new Date(item.date).toLocaleDateString("zh-CN", {
      month: "numeric",
      day: "numeric",
    }),
    requests: item.total_requests,
    errors: item.total_errors,
    latency: Math.round(item.avg_latency_ms),
  }));

  return (
    <div className="signal-panel p-5">
      <div className="mb-3 flex items-start justify-between gap-4">
        <div>
          <p className="panel-header-label">Traffic Trend</p>
          <h3 className="mt-1 text-xl font-semibold tracking-[-0.04em] text-foreground">
            最近 7 天请求走势
          </h3>
        </div>
        <div className="flex flex-shrink-0 items-center gap-4 pt-1 text-xs text-slate-500">
          <div className="flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-sm bg-primary" />
            <span>请求数</span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-sm bg-destructive" />
            <span>错误数</span>
          </div>
        </div>
      </div>

      <ResponsiveContainer width="100%" height={180}>
        <BarChart data={chartData} barSize={24} barGap={4} barCategoryGap={16}>
          <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" vertical={false} />
          <XAxis 
            dataKey="date" 
            stroke="hsl(var(--muted-foreground))"
            tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 12 }}
            tickLine={{ stroke: "hsl(var(--border))" }}
            axisLine={{ stroke: "hsl(var(--border))" }}
          />
          <YAxis 
            stroke="hsl(var(--muted-foreground))"
            tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 12 }}
            tickLine={{ stroke: "hsl(var(--border))" }}
            axisLine={{ stroke: "hsl(var(--border))" }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: "hsl(var(--card))",
              border: "1px solid hsl(var(--border))",
              borderRadius: "var(--radius)",
              color: "hsl(var(--foreground))",
            }}
          />
          <Bar
            dataKey="requests"
            fill="hsl(var(--chart-1))"
            name="请求数"
            radius={[6, 6, 0, 0]}
          />
          <Bar
            dataKey="errors"
            fill="hsl(var(--destructive))"
            name="错误数"
            radius={[6, 6, 0, 0]}
          />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
