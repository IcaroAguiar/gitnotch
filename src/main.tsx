import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("Elemento raiz ausente.");

createRoot(root).render(
  <StrictMode>
    <main>
      <span className="wordmark">Git Notch</span>
      <h1>Um lugar para revisar suas mudanças.</h1>
      <p>
        O aplicativo está em desenvolvimento. A leitura de repositórios ainda
        não está disponível nesta versão.
      </p>
      <footer>Prévia de desenvolvimento · 0.1.0</footer>
    </main>
  </StrictMode>,
);
