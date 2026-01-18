import { invoke } from "@tauri-apps/api/core";
import { onMounted, ref } from "vue";

export type Theme = "dark" | "light";

const currentTheme = ref<Theme>("dark");

export function useTheme() {
  async function setTheme(theme: Theme) {
    try {
      await invoke("update_config", {
        configJson: {
          theme: theme,
        },
      });
      currentTheme.value = theme;
      document.documentElement.setAttribute("data-theme", theme);
    } catch (error) {
      console.error("Failed to update theme:", error);
    }
  }

  function toggleTheme() {
    const newTheme = currentTheme.value === "dark" ? "light" : "dark";
    setTheme(newTheme);
  }

  async function loadTheme() {
    try {
      const config = await invoke<{
        theme: string;
      }>("get_config");
      const theme = (config.theme || "dark") as Theme;
      currentTheme.value = theme;
      document.documentElement.setAttribute("data-theme", theme);
    } catch (error) {
      console.error("Failed to load theme:", error);
      // Default to dark theme
      document.documentElement.setAttribute("data-theme", "dark");
    }
  }

  onMounted(loadTheme);

  return {
    theme: currentTheme,
    setTheme,
    toggleTheme,
    loadTheme,
  };
}
