import { cn } from "@/lib/utils";

interface EmptyStateProps {
  icon: React.ReactNode;
  title: string;
  description: string;
  action?: React.ReactNode;
  className?: string;
}

export function EmptyState({ icon, title, description, action, className }: EmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center py-16 text-center", className)}>
      <div className="relative mb-8">
        <div className="relative p-8 border border-slate-200 bg-slate-50">
          <div className="absolute inset-0 bg-gradient-to-br from-[#00dc82]/5 to-transparent" />
          <div className="text-[#00dc82] relative z-10">{icon}</div>
        </div>
        <div className="absolute -top-1 -left-1 w-2.5 h-2.5 border-l-2 border-t-2 border-[#00dc82]" />
        <div className="absolute -bottom-1 -right-1 w-2.5 h-2.5 border-r-2 border-b-2 border-[#00dc82]" />
      </div>

      <h3 className="mb-2 text-xl font-semibold text-slate-900">{title}</h3>
      <p className="mb-6 max-w-sm text-slate-600">{description}</p>
      {action}
    </div>
  );
}
