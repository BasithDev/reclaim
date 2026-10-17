// Reclaim: root component. Applies the theme, loads settings once, then renders the app shell.
import { useEffect } from "react";
import { AppShell } from "./shell/AppShell";
import { applyCachedTheme, useSettingsStore } from "./store/settings";

applyCachedTheme();

export default function App() {
  useEffect(() => { void useSettingsStore.getState().load(); }, []);
  return <AppShell />;
}
