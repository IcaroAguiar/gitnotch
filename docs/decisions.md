# Decisões de implementação

## Aplicação mínima

O aplicativo começa com uma janela Tauri convencional e uma tela de desenvolvimento. A estratégia de notch e drawer será validada separadamente, conforme a especificação. A tela inicial não lê repositórios nem solicita acesso ao filesystem.

React e Vite produzem assets locais em dist. Rust incorpora esses assets no executável. Não há servidor HTTP, Node em produção, fontes remotas ou chamadas de API. A configuração CSP bloqueia conexões do frontend e nenhuma capability nativa é concedida nesta etapa. Comandos customizados também não estão registrados.

O fluxo inicial usa builds pontuais. Nenhum servidor persistente de desenvolvimento é necessário. Um futuro fluxo com Vite persistente deve respeitar as regras de Portly do ambiente do mantenedor; não é pré-requisito de execução do aplicativo distribuído.

As ferramentas têm versões fixadas nos manifests e arquivos de toolchain. pnpm-lock.yaml e src-tauri/Cargo.lock registram as resoluções. A CI verifica builds nos três sistemas, mas build aprovado não comprova interação nativa ou suporte completo.

Referências consultadas:

- [Tauri com Vite](https://v2.tauri.app/start/frontend/vite/).
- [Configuração de frontend Tauri](https://v2.tauri.app/start/frontend/).
- [Permissões Tauri](https://v2.tauri.app/security/permissions/).

O ícone provisório tem fonte em `src-tauri/icons/source.svg`. Para regenerar, execute `pnpm exec tauri icon src-tauri/icons/source.svg --output .local/generated-icons` e copie somente icon.png, icon.ico e icon.icns para src-tauri/icons. Os demais formatos gerados não pertencem a este aplicativo desktop.
