// Reclaim: React entry point; mounts <App /> and loads global styles.
import React from "react";
import ReactDOM from "react-dom/client";
import { MotionConfig } from "motion/react";
import App from "./App";
import "./styles/global.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MotionConfig reducedMotion="user">
      <App />
    </MotionConfig>
  </React.StrictMode>,
);
