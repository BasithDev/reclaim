// Reclaim: React entry point; mounts <App /> and loads global styles. With VITE_MOCK=1 it installs the mock backend first.
import React from "react";
import ReactDOM from "react-dom/client";
import { MotionConfig } from "motion/react";
import "./styles/global.css";

async function boot() {
  if (import.meta.env.VITE_MOCK === "1") {
    const { installMockBackend } = await import("./lib/mock");
    installMockBackend();
  }
  // Imported after the mock is installed: store/scan.ts subscribes to backend events at module load.
  const { default: App } = await import("./App");
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <MotionConfig reducedMotion="user">
        <App />
      </MotionConfig>
    </React.StrictMode>,
  );
}
void boot();
