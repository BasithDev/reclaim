// Reclaim: Button and IconButton, the standard clickable controls (variants, sizes, loading state).
import type { ButtonHTMLAttributes, ReactNode } from "react";
import type { LucideIcon } from "lucide-react";
import { Loader2 } from "lucide-react";
import { cn } from "./cn";
import { Tooltip } from "./Tooltip";

type Variant = "primary" | "secondary" | "ghost" | "danger";
type Size = "sm" | "md" | "lg";

const variants: Record<Variant, string> = {
  primary: "bg-accent text-accent-contrast hover:bg-accent-strong shadow-sm",
  secondary: "bg-surface text-text border border-border hover:bg-surface-2 hover:border-border-strong shadow-sm",
  ghost: "text-muted hover:bg-surface-2 hover:text-text",
  danger: "bg-danger text-white hover:opacity-90 shadow-sm",
};
const sizes: Record<Size, string> = {
  sm: "h-7 px-2.5 text-xs gap-1.5 rounded-md",
  md: "h-9 px-3.5 text-sm gap-2 rounded-md",
  lg: "h-11 px-5 text-sm gap-2 rounded-lg",
};
const iconSizes: Record<Size, string> = { sm: "h-7 w-7 rounded-md", md: "h-9 w-9 rounded-md", lg: "h-11 w-11 rounded-lg" };
const iconPx: Record<Size, number> = { sm: 14, md: 16, lg: 18 };

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
  icon?: LucideIcon;
  loading?: boolean;
}

export function Button({ variant = "secondary", size = "md", icon: Icon, loading, disabled, className, children, type = "button", ...rest }: ButtonProps) {
  return (
    <button
      type={type}
      disabled={disabled || loading}
      className={cn(
        "inline-flex shrink-0 cursor-pointer items-center justify-center whitespace-nowrap font-medium transition-all duration-150 active:scale-[0.97] disabled:pointer-events-none disabled:opacity-50",
        variants[variant], sizes[size], className,
      )}
      {...rest}
    >
      {loading ? <Loader2 size={iconPx[size]} className="animate-spin" /> : Icon ? <Icon size={iconPx[size]} /> : null}
      {children as ReactNode}
    </button>
  );
}

export interface IconButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> {
  icon: LucideIcon;
  label: string;
  variant?: Variant;
  size?: Size;
}

export function IconButton({ icon: Icon, label, variant = "ghost", size = "md", className, type = "button", ...rest }: IconButtonProps) {
  return (
    <Tooltip content={label}>
      <button
        type={type}
        aria-label={label}
        className={cn(
          "inline-flex shrink-0 cursor-pointer items-center justify-center transition-all duration-150 active:scale-95 disabled:pointer-events-none disabled:opacity-50",
          variants[variant], iconSizes[size], className,
        )}
        {...rest}
      >
        <Icon size={iconPx[size]} />
      </button>
    </Tooltip>
  );
}
