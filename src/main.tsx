import React from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import WheelOverlay from "./components/WheelOverlay";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("SHIFT could not find its application root.");

const isWheelOverlay = new URLSearchParams(window.location.search).get("wheel") === "1";
if (isWheelOverlay) document.documentElement.dataset.wheelOverlay = "true";

createRoot(root).render(
  <React.StrictMode>
    {isWheelOverlay ? <WheelOverlay /> : <App />}
  </React.StrictMode>,
);
