import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { DrawerRoot } from "./drawer/DrawerRoot";
import { NotchTab } from "./notch/NotchTab";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("Elemento raiz ausente.");

const { label } = getCurrentWebviewWindow();

const entry =
  label === "notch" ? <NotchTab /> : label === "drawer" ? <DrawerRoot /> : null;

if (!entry) throw new Error(`Janela desconhecida: ${label}`);

document.documentElement.dataset.window = label;

createRoot(root).render(<StrictMode>{entry}</StrictMode>);
