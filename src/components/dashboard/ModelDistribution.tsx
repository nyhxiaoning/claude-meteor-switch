import {
  Bar,
  BarChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

interface ModelBreakdown {
  model: string;
  requests: number;
  input_tokens: number;
  output_tokens: number;
}

interface ModelDistributionProps {
  data: ModelBreakdown[];
}

export function ModelDistribution({ data }: ModelDistributionProps) {
  if (data.length === 0) {
    return (
      <div className="signal-panel flex h-56 items-center justify-center">
        <p className="text-sm text-muted-foreground">暂无数据</p>
      </div>
    );
  }

  const chartData = data.map((item) => ({
    model: item.model.replace("claude-", "").replace("-4-6", "").replace("-4-5", ""),
    requests: item.requests,
  }));

  return (
    <div className="signal-panel p-5">
      <div className="mb-3">
        <p className="panel-header-label">Model Pressure</p>
        <h3 className="mt-1 text-lg font-semibold tracking-normal text-foreground">
          模型请求分布
        </h3>
      </div>

      <ResponsiveContainer width="100%" height={190}>
        <BarChart data={chartData} layout="vertical">
          <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" horizontal={false} />
          <XAxis 
            type="number" 
            stroke="hsl(var(--muted-foreground))"
            tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 12 }}
          />
          <YAxis
            dataKey="model"
            type="category"
            width={80}
            stroke="hsl(var(--muted-foreground))"
            tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 12 }}
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
            fill="hsl(var(--chart-2))"
            name="请求数"
            radius={[0, 6, 6, 0]}
          />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
