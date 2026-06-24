import { emit } from "@tauri-apps/api/event";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type ThemeMode = "light" | "dark";

export const THEME_STORAGE_KEY = "traylink-theme";
export const THEME_CHANGED_EVENT = "theme-changed";

const WINDOW_COLORS: Record<ThemeMode, { red: number; green: number; blue: number; alpha: number }> =
  {
    dark: { red: 15, green: 23, blue: 42, alpha: 255 },
    light: { red: 255, green: 255, blue: 255, alpha: 255 },
  };

export function getStoredTheme(): ThemeMode {
  if (typeof localStorage === "undefined") {
    return "dark";
  }

  const value = localStorage.getItem(THEME_STORAGE_KEY);
  return value === "light" ? "light" : "dark";
}

export function applyTheme(theme: ThemeMode, root: HTMLElement = document.documentElement) {
  root.classList.add("traylink-theme");
  root.classList.toggle("dark", theme === "dark");
}

export function initTheme() {
  const theme = getStoredTheme();
  applyTheme(theme);
  void syncWindowBackground(theme);
}

export async function syncWindowBackground(theme: ThemeMode) {
  if (!isTauri()) {
    return;
  }

  try {
    await getCurrentWindow().setBackgroundColor(WINDOW_COLORS[theme]);
  } catch {
    // Optional — window background may be unavailable in dev/browser.
  }
}

export async function setTheme(theme: ThemeMode) {
  localStorage.setItem(THEME_STORAGE_KEY, theme);
  applyTheme(theme);
  await syncWindowBackground(theme);

  if (isTauri()) {
    await emit(THEME_CHANGED_EVENT, theme);
  }
}

export async function toggleTheme(current: ThemeMode): Promise<ThemeMode> {
  const next = current === "dark" ? "light" : "dark";
  await setTheme(next);
  return next;
}
