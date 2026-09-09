# Validação da aplicação mínima

Ambiente local: macOS 26.6.2, Apple Silicon, Node 22.23.2, pnpm 10.32.1 e Rust 1.98.1.

## Verificações locais

- `pnpm check`: lint, typecheck e dois testes do contrato de configuração passaram.
- `pnpm build`: assets estáticos produzidos em dist.
- `pnpm rust:check`: rustfmt e Clippy sem warnings; test runner Rust concluiu com zero testes. Não há lógica de domínio nesta etapa.
- `pnpm desktop:bundle --bundles app -- --locked`: pacote macOS gerado.
- Abertura pelo LaunchServices: janela Git Notch exibiu a mensagem de desenvolvimento. O WebView carregou `tauri://localhost`, sem servidor Vite. Captura nativa mantida como artefato local, fora do Git.

## Reproduzir

Execute os comandos do README, abra o pacote macOS pelo Finder e confira o título, a mensagem de desenvolvimento e o layout sem cortes. Encerre pelo menu do aplicativo ou Cmd+Q. Reabra o pacote sem servidor de desenvolvimento e sem conexão com a internet.

## Limites da evidência

O teste de execução com conexões IP bloqueadas por sandbox-exec foi inconclusivo. A instância iniciou, mas não pôde ser inspecionada pelo controle de aplicativos. O teste automatizado confirma CSP sem conexões de frontend; isso não substitui a verificação visual offline do pacote completo.

A CI compila nas plataformas configuradas, sem executar suas interfaces. Windows, Linux, fallback visual, notch, gaveta e desempenho ainda não foram validados nativamente. O pacote local não é uma release assinada/notarizada para distribuição.

O requisito de abertura visual offline permanece pendente de confirmação. A aplicação mínima pode ser revisada, mas esse critério ainda não deve ser marcado como aprovado.
