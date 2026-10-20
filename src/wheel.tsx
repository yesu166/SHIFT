import React from "react";
import { createRoot } from "react-dom/client";
import WheelOverlay from "./components/WheelOverlay";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("SHIFT could not find its wheel root.");

document.documentElement.dataset.wheelOverlay = "true";

createRoot(root).render(<React.StrictMode><WheelOverlay /></React.StrictMode>);
