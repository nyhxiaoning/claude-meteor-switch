"use client"

import * as React from "react"
import { cn } from "@/lib/utils"

interface SwitchProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'size'> {
  size?: "sm" | "default"
}

function Switch({
  className,
  size = "default",
  ...props
}: SwitchProps) {
  const checked = Boolean(props.checked)
  const disabled = Boolean(props.disabled)
  const isSmall = size === "sm"

  return (
    <label
      className={cn(
        "relative inline-flex items-center transition-all",
        disabled ? "cursor-not-allowed" : "cursor-pointer",
        className
      )}
    >
      <input
        type="checkbox"
        role="switch"
        aria-checked={checked}
        className="peer sr-only"
        {...props}
      />
      <div
        className={cn(
          "relative border transition-[background-color,border-color,box-shadow] duration-200 ease-out",
          "peer-focus-visible:ring-2 peer-focus-visible:ring-primary/30 peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-background",
          isSmall ? "h-5 w-9 rounded-full" : "h-6 w-11 rounded-full",
          checked
            ? "border-primary bg-primary shadow-[inset_0_0_0_1px_rgba(0,0,0,0.04)]"
            : "border-slate-300 bg-slate-100",
          !disabled && checked && "hover:border-[#00b870] hover:bg-[#00b870]",
          !disabled && !checked && "hover:border-slate-400 hover:bg-slate-50",
          disabled && "border-slate-200 bg-slate-100 opacity-60"
        )}
      >
        <div
          className={cn(
            "absolute left-0.5 top-1/2 -translate-y-1/2 rounded-full bg-white shadow-sm transition-transform duration-200 ease-out",
            "border border-black/5",
            isSmall ? "h-4 w-4" : "h-5 w-5",
            checked
              ? isSmall
                ? "translate-x-4 -translate-y-1/2"
                : "translate-x-5 -translate-y-1/2"
              : "translate-x-0 -translate-y-1/2",
            disabled && "shadow-none"
          )}
        />
      </div>
    </label>
  )
}

export { Switch }
