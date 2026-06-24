import { Moon, Sun } from "lucide-react";
import { useTheme } from "@/hooks/useTheme";
import { cn } from "@/lib/utils";

type ThemeToggleProps = {
  compact?: boolean;
  className?: string;
};

export function ThemeToggle({ compact = false, className }: ThemeToggleProps) {
  const { theme, toggle, isDark } = useTheme();

  return (
    <button
      type="button"
      onClick={() => void toggle()}
      aria-label={isDark ? "Chuyển sang giao diện sáng" : "Chuyển sang giao diện tối"}
      title={isDark ? "Light mode" : "Dark mode"}
      className={cn(
        "inline-flex items-center justify-center rounded-lg border border-border bg-background/60 text-muted-foreground transition hover:bg-accent hover:text-accent-foreground",
        compact ? "size-7" : "size-9",
        className,
      )}
    >
      {isDark ? <Sun className="size-3.5" /> : <Moon className="size-3.5" />}
      {!compact && (
        <span className="sr-only">{theme === "dark" ? "Light" : "Dark"}</span>
      )}
    </button>
  );
}

type ThemeModeSwitchProps = {
  className?: string;
};

export function ThemeModeSwitch({ className }: ThemeModeSwitchProps) {
  const { theme, selectTheme } = useTheme();

  return (
    <div className={cn("flex gap-2", className)}>
      <button
        type="button"
        onClick={() => void selectTheme("light")}
        className={cn(
          "inline-flex flex-1 items-center justify-center gap-1.5 rounded-lg border px-3 py-2 text-sm font-medium transition",
          theme === "light"
            ? "border-primary bg-primary/10 text-primary"
            : "border-border text-muted-foreground hover:bg-accent",
        )}
      >
        <Sun className="size-3.5" />
        Light
      </button>
      <button
        type="button"
        onClick={() => void selectTheme("dark")}
        className={cn(
          "inline-flex flex-1 items-center justify-center gap-1.5 rounded-lg border px-3 py-2 text-sm font-medium transition",
          theme === "dark"
            ? "border-primary bg-primary/10 text-primary"
            : "border-border text-muted-foreground hover:bg-accent",
        )}
      >
        <Moon className="size-3.5" />
        Dark
      </button>
    </div>
  );
}
