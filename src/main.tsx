import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { DesktopRoot } from "./desktop/DesktopRoot";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("Elemento raiz ausente.");

createRoot(root).render(
  <StrictMode>
    <DesktopRoot />
  </StrictMode>,
);
