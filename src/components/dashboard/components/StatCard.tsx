
import { Card, CardContent, CardHeader } from "@/components/ui/card";

interface StatCardProps {
  icon: React.ReactNode;
  label: string;
  value: number | string;
  color?: "primary" | "secondary" | "accent" | "destructive";
}

const colorClasses = {
  primary: "text-primary bg-primary/10",
  secondary: "text-blue-600 bg-blue-100",
  accent: "text-amber-600 bg-amber-100",
  destructive: "text-destructive bg-destructive/10",
};

export function StatCard({ icon, label, value, color = "primary" }: StatCardProps) {
  const colorClass = colorClasses[color];
  
  return (
    <Card className="hover-lift">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <span className="text-sm font-medium text-muted-foreground">{label}</span>
        <div className={`p-2 rounded-md ${colorClass}`}>
          {icon}
        </div>
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-semibold">{value}</div>
      </CardContent>
    </Card>
  );
}
