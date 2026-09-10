# Validação da aba e da gaveta nativas (GN-01B)

Ambiente local: macOS 26.6.2, Apple Silicon, display 1920 × 1080 sem Retina, Node 22.23.2, pnpm 10.32.1 e Rust 1.98.1.

Código verificado: `2578cc0`. Alterações posteriores deste registro são apenas documentação.

## Verificações locais

- `pnpm check`: lint, typecheck e três testes do contrato de configuração passaram.
- `pnpm build`: assets estáticos produzidos em `dist`.
- `pnpm rust:check`: rustfmt e Clippy sem warnings; 19 testes passaram, sendo 9 do módulo `desktop` e 10 do leitor Git.
- `pnpm desktop:build -- --locked`: binário release produzido em `src-tauri/target/release/gitnotch`.
- `pnpm desktop:bundle --bundles app -- --locked`: `Git Notch.app` produzido em `src-tauri/target/release/bundle/macos/`.

## Cenários nativos comprovados

Build de release aberto pelo Finder, com o T3 Code atrás e o Git Notch ativo.

- Geometria: a aba abriu em 28 × 64 na borda direita da área útil, centralizada na vertical (posição 1892, 481 no display de 1920 × 1080). A gaveta abriu em 600 × 800 com a borda direita a 12 px da aba e o centro alinhado a ela (posição 1280, 113).
- Abrir e recolher por clique: clique real na aba abriu a gaveta reutilizada; novo clique com a gaveta em foco recolheu sem reabrir, exercitando o colapso por perda de foco coalescido com o clique na aba.
- 20 ciclos de abrir e recolher: a contagem de janelas se manteve em uma com a gaveta recolhida e duas com ela aberta, sem duplicatas e sem acumular views. A CPU em repouso ficou em 0,0% em cinco amostras de dois segundos.
- Foco preservado: com a aba visível e a gaveta recolhida, a digitação contínua em um documento do TextEdit não foi interrompida.
- Hit-testing: cliques reais em três pontos vizinhos à aba (à esquerda, acima e abaixo) foram entregues à janela do T3 Code atrás, sem camada invisível capturando a região. As áreas clicáveis reais são as duas janelas registradas em AX, 28 × 64 para a aba e 600 × 800 para a gaveta; as margens transparentes internas à janela da aba pertencem ao alvo de 28 × 64 previsto na especificação.
- Vídeo local em `artifacts/gn-01b/08-ciclos-abrir-fechar.mov` registra três ciclos de abrir e recolher. Capturas locais acompanham o roteiro.

## Memória

Medida com a árvore do processo mais os auxiliares WebKit. No macOS os auxiliares são reparentados ao launchd (`ppid` 1), então a atribuição foi feita por diferença de PIDs antes e depois de abrir o aplicativo. O roteiro original do plano soma apenas filhos diretos e por isso foi complementado. Duas métricas foram registradas: soma de RSS e `phys_footprint` por processo.

| Estado | Soma de RSS | phys_footprint |
|---|---|---|
| Nunca aberto, logo após abrir | 189,2 MiB | 85,1 MiB |
| Gaveta aberta | 163,8 MiB | 80,6 a 134,8 MiB |
| Recolhido após 20 ciclos | 135,6 MiB | 133,6 MiB |

O RSS soma páginas compartilhadas em cada processo e superestima o custo. O `phys_footprint` do processo principal ficou em 22 a 24 MB e os auxiliares variaram com o cache de composição do GPU. Nenhuma métrica mostrou crescimento com a gaveta aberta nem depois de 20 ciclos. O orçamento inicial de 150 MiB recolhido da seção 16.4 não foi comparado com metodologia própria; a medição comparável fica para GN-08A.

## Reproduzir

```sh
pnpm install --frozen-lockfile
pnpm check
pnpm build
pnpm rust:check
pnpm desktop:build -- --locked
pnpm desktop:bundle --bundles app -- --locked
open "src-tauri/target/release/bundle/macos/Git Notch.app"
```

Com o aplicativo aberto, a aba fica na borda direita da área útil. O toggle também pode ser reproduzido sem clique físico, pelo botão exposto em acessibilidade (com a gaveta recolhida, `window 1` é a aba; com ela aberta, a aba é `window 2`):

```sh
osascript -e 'tell application "System Events" to tell process "Git Notch" to click button "Abrir a gaveta do Git Notch" of UI element 1 of scroll area 1 of group 1 of group 1 of window 1'
```

Cliques físicos no roteiro local usaram `artifacts/gn-01b/click.swift`, que publica eventos de mouse por `CGEvent`. A posição e o tamanho das janelas podem ser conferidos em AX com `get {position, size} of every window`. As medições de memória usaram `artifacts/gn-01b/snap.py`, `mem.py` e `fp.py`.

## Limites da evidência

- Somente macOS 26.6.2 em Apple Silicon foi validado nativamente. Windows, Linux, X11, Wayland e Intel não foram testados.
- Segundo monitor com DPI diferente, remoção de monitor, Spaces e fullscreen não foram testados.
- Material, blur e vibrancy ficam para GN-01C; animação calibrada, `Esc`, menus, "manter aberta" e persistência de posição ficam para GN-07.
- Ao recolher a gaveta pelo clique na aba, o foco não retorna automaticamente ao aplicativo anterior. O comportamento fica pendente para GN-07B.
- A medição rodou com outra instância do Git Notch em execução em outro checkout. A atribuição de auxiliares usou diferença de PIDs; um auxiliar de rede transitório observado na primeira medição pode pertencer ao conjunto compartilhado do WebKit.
- O critério de abertura visual offline herdado do GN-01A continua pendente e não é resolvido por este ticket.
