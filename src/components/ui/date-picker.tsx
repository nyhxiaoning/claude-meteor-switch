import { useMemo, useState } from "react";
import { Popover } from "@base-ui/react/popover";
import { CalendarDays, ChevronLeft, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";

interface DatePickerProps {
  id?: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}

const weekDays = ["一", "二", "三", "四", "五", "六", "日"];

function parseDate(value: string) {
  if (!value) return null;
  const [year, month, day] = value.split("-").map(Number);
  if (!year || !month || !day) return null;
  return new Date(year, month - 1, day);
}

function toDateValue(date: Date) {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function formatDate(value: string) {
  const date = parseDate(value);
  if (!date) return "";
  return `${date.getMonth() + 1}月${date.getDate()}日`;
}

function getCalendarDays(monthDate: Date) {
  const year = monthDate.getFullYear();
  const month = monthDate.getMonth();
  const firstDay = new Date(year, month, 1);
  const firstWeekday = (firstDay.getDay() + 6) % 7;
  const startDate = new Date(year, month, 1 - firstWeekday);

  return Array.from({ length: 42 }, (_, index) => {
    const date = new Date(startDate);
    date.setDate(startDate.getDate() + index);
    return date;
  });
}

export function DatePicker({
  id,
  value,
  onChange,
  placeholder = "选择日期",
  className,
}: DatePickerProps) {
  const selectedDate = parseDate(value);
  const [open, setOpen] = useState(false);
  const [visibleMonth, setVisibleMonth] = useState(() => selectedDate ?? new Date());

  const days = useMemo(() => getCalendarDays(visibleMonth), [visibleMonth]);
  const todayValue = toDateValue(new Date());
  const selectedValue = selectedDate ? toDateValue(selectedDate) : "";

  const moveMonth = (offset: number) => {
    setVisibleMonth(new Date(visibleMonth.getFullYear(), visibleMonth.getMonth() + offset, 1));
  };

  const selectDate = (date: Date) => {
    onChange(toDateValue(date));
    setVisibleMonth(date);
    setOpen(false);
  };

  return (
    <Popover.Root open={open} onOpenChange={(nextOpen) => setOpen(nextOpen)}>
      <Popover.Trigger
        id={id}
        className={cn(
          "inline-flex h-8 w-[150px] items-center justify-between gap-2 rounded-lg border border-input bg-background px-2.5 text-xs text-slate-700 transition-colors outline-none hover:bg-slate-50 focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50",
          !value && "text-slate-400",
          className,
        )}
      >
        <span className="truncate">{value ? formatDate(value) : placeholder}</span>
        <CalendarDays className="h-3.5 w-3.5 flex-shrink-0 text-slate-400" />
      </Popover.Trigger>

      <Popover.Portal>
        <Popover.Positioner sideOffset={6} align="start" className="z-50">
          <Popover.Popup className="w-[248px] rounded-lg border border-slate-200 bg-white p-3 shadow-lg outline-none data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95">
            <div className="mb-3 flex items-center justify-between">
              <button
                type="button"
                className="flex h-7 w-7 items-center justify-center rounded-md text-slate-500 hover:bg-slate-100 hover:text-slate-950"
                onClick={() => moveMonth(-1)}
              >
                <ChevronLeft className="h-4 w-4" />
              </button>
              <div className="text-sm font-semibold text-slate-950">
                {visibleMonth.getFullYear()}年{visibleMonth.getMonth() + 1}月
              </div>
              <button
                type="button"
                className="flex h-7 w-7 items-center justify-center rounded-md text-slate-500 hover:bg-slate-100 hover:text-slate-950"
                onClick={() => moveMonth(1)}
              >
                <ChevronRight className="h-4 w-4" />
              </button>
            </div>

            <div className="grid grid-cols-7 gap-1 text-center">
              {weekDays.map((day) => (
                <div key={day} className="py-1 text-[11px] font-semibold text-slate-400">
                  {day}
                </div>
              ))}
              {days.map((date) => {
                const dateValue = toDateValue(date);
                const isCurrentMonth = date.getMonth() === visibleMonth.getMonth();
                const isSelected = dateValue === selectedValue;
                const isToday = dateValue === todayValue;

                return (
                  <button
                    key={dateValue}
                    type="button"
                    className={cn(
                      "flex h-8 items-center justify-center rounded-md text-xs transition-colors",
                      isCurrentMonth ? "text-slate-700" : "text-slate-300",
                      isToday && "font-bold text-primary",
                      isSelected
                        ? "bg-primary text-white hover:bg-primary/90"
                        : "hover:bg-slate-100 hover:text-slate-950",
                    )}
                    onClick={() => selectDate(date)}
                  >
                    {date.getDate()}
                  </button>
                );
              })}
            </div>

            <div className="mt-3 flex items-center justify-between border-t border-slate-100 pt-3">
              <button
                type="button"
                className="text-xs font-medium text-slate-500 hover:text-slate-950"
                onClick={() => {
                  onChange("");
                  setOpen(false);
                }}
              >
                清空
              </button>
              <button
                type="button"
                className="text-xs font-semibold text-primary hover:text-[#00b870]"
                onClick={() => selectDate(new Date())}
              >
                今天
              </button>
            </div>
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  );
}
