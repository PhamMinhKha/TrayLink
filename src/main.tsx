import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { UsagePopup } from "./components/UsagePopup";
import { initTheme } from "./lib/theme";
import "./index.css";
import "@fortawesome/fontawesome-free/css/all.min.css";

initTheme();

const label = getCurrentWindow().label;

if (label === "usage-popup") {
  document.documentElement.classList.add("usage-popup-page");
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {label === "usage-popup" ? <UsagePopup /> : <App />}
  </React.StrictMode>,
);
