# Validação

- [Aplicação mínima](validation/minimal-desktop.md). Build e janela macOS verificados; teste visual offline ainda pendente.
- [Executor Git de leitura e segurança](validation/git-reader.md). Subprocessos, formato porcelain v2, preflight de filtros e prova de não-mutação verificados.
- [Autorização de raízes e preferências](validation/GN-03A.md). Escrita atômica, época de autorização, ids exauridos, rejeição de caminho não UTF-8 e IPC com caminho arbitrário são cobertos por verificações automatizadas; a QA nativa da gaveta integrada permanece pendente.
- [Aba e gaveta nativas](validation/GN-01B.md). Geometria, material e ciclos do drawer flutuante verificados em QA nativa limitada no macOS; hover físico, foco/hit-testing entre aplicativos, acessibilidade e monitores reais permanecem pendentes. Windows e Linux não testados.

Builds de CI não comprovam interação nativa. Planejamento e registros administrativos permanecem locais.
