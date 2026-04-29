import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";

interface ProviderBreakdown {
  provider_name: string;
  requests: number;
  input_tokens: number;
  output_tokens: number;
}

interface ProviderDistributionProps {
  data: ProviderBreakdown[];
}

const COLORS = [
  "hsl(var(--chart-1))",
  "hsl(var(--chart-2))",
  "hsl(var(--chart-3))",
  "hsl(var(--chart-4))",
  "hsl(var(--chart-5))",
];

export function ProviderDistribution({ data }: ProviderDistributionProps) {
  if (data.length === 0) {
    return (
      <div className="signal-panel flex h-56 items-center justify-center">
        <p className="text-sm text-muted-foreground">暂无数据</p>
      </div>
    );
  }

  const chartData = data.map((item) => ({
    name: item.provider_name,
    value: item.requests,
  }));

  return (
    <div className="signal-panel p-5">
      <div className="mb-3">
        <p className="panel-header-label">Provider Mix</p>
        <h3 className="mt-1 text-lg font-semibold tracking-normal text-foreground">
          厂商请求分布
        </h3>
      </div>

      <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_160px]">
        <ResponsiveContainer width="100%" height={190}>
          <PieChart>
            <Pie
              data={chartData}
              cx="50%"
              cy="50%"
              labelLine={false}
              label={({ percent }) => `${(percent * 100).toFixed(0)}%`}
              outerRadius={70}
              innerRadius={36}
              fill="hsl(var(--primary))"
              dataKey="value"
              stroke="hsl(var(--background))"
              strokeWidth={2}
            >
              {chartData.map((_, index) => (
                <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
              ))}
            </Pie>
            <Tooltip
              contentStyle={{
                backgroundColor: "hsl(var(--card))",
                border: "1px solid hsl(var(--border))",
                borderRadius: "var(--radius)",
                color: "hsl(var(--foreground))",
              }}
            />
          </PieChart>
        </ResponsiveContainer>

        <div className="space-y-1.5 self-center">
          {chartData.map((item, index) => (
            <div
              key={item.name}
              className="flex items-center justify-between gap-2 border-b border-slate-100 py-1.5 last:border-b-0"
            >
              <div className="flex min-w-0 items-center gap-2">
                <span
                  className="h-2.5 w-2.5 rounded-full"
                  style={{ backgroundColor: COLORS[index % COLORS.length] }}
                />
                <p className="truncate text-xs font-medium text-foreground">{item.name}</p>
              </div>
              <p className="flex-shrink-0 font-mono text-xs text-muted-foreground">{item.value}</p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
