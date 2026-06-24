import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { isTauri } from "@tauri-apps/api/core";
import {
  applyTheme,
  getStoredTheme,
  setTheme,
  syncWindowBackground,
  THEME_CHANGED_EVENT,
  THEME_STORAGE_KEY,
  toggleTheme,
  type ThemeMode,
} from "@/lib/theme";

export function useTheme() {
  const [theme, setThemeState] = useState<ThemeMode>(() => getStoredTheme());

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }

    let unlisten: (() => void) | undefined;

    void listen<ThemeMode>(THEME_CHANGED_EVENT, (event) => {
      const next = event.payload === "light" ? "light" : "dark";
      localStorage.setItem(THEME_STORAGE_KEY, next);
      setThemeState(next);
      applyTheme(next);
      void syncWindowBackground(next);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const selectTheme = useCallback(async (next: ThemeMode) => {
    await setTheme(next);
    setThemeState(next);
  }, []);

  const toggle = useCallback(async () => {
    const next = await toggleTheme(theme);
    setThemeState(next);
  }, [theme]);

  return { theme, selectTheme, toggle, isDark: theme === "dark" };
}
