# Validação da aba e da gaveta nativas (GN-01B)

Ambiente local: macOS 26.6.2, Apple Silicon, display 1920 × 1080 sem Retina, Node 22.23.2, pnpm 10.32.1 e Rust 1.98.1.

Código verificado: `cb8883c`. Alterações posteriores deste registro são apenas documentação.

## Verificações locais

- `pnpm check`: lint, typecheck e quatro testes do contrato de configuração passaram.
- `pnpm build`: assets estáticos produzidos em `dist`.
- `pnpm rust:check`: rustfmt e Clippy sem warnings; 20 testes passaram, sendo 10 do módulo `desktop` e 10 do leitor Git.
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

## Movimento e transições

A transição é de conteúdo, não de geometria nativa. A janela da gaveta permanece na posição final e o Rust não emite `set_size` ou `set_position` por frame, conforme a seção 6. O webview anima opacidade e um deslocamento curto com escala, o que entrega o reveal sem prometer morph entre as duas janelas.

| Elemento | Movimento | Curva |
|---|---|---|
| Hover da aba | realce e deslocamento de 1 px em 110 ms | `cubic-bezier(0.2, 0.8, 0.2, 1)` |
| Abrir a gaveta | opacidade, 14 px e escala 0,985 em 220 ms | `cubic-bezier(0.2, 0.8, 0.2, 1)` |
| Recolher a gaveta | reverso em 170 ms; a janela é ocultada 180 ms depois | `cubic-bezier(0.4, 0, 1, 1)` |
| Movimento reduzido | fade de opacidade em 80 ms, sem deslocamento | `linear` |

A aba fica grudada na borda direita, com cantos arredondados apenas no lado exposto e raio de 12. A gaveta usa raio de 20. Onde `corner-shape: superellipse()` estiver disponível, a curvatura vira squircle; o WKWebView do macOS 26.6 ainda não suporta a propriedade, então o build testado usa o arredondamento circular do `border-radius`.

O recolhimento atrasado é protegido pela geração. O Rust emite `gitnotch://drawer-state` com o estado e a geração, espera 180 ms em uma thread curta e só oculta a janela se a geração ainda for a mais recente. Uma nova abertura nesse intervalo cancela o fechamento pendente. A gaveta recebe uma capability própria com apenas `core:event:allow-listen` e `core:event:allow-unlisten`.

Evidência do movimento: o vídeo local `artifacts/gn-01b/13-motion-abrir-fechar.mov` registra três ciclos com o reveal, a captura `10-aba-grudada-limpa.png` e o recorte `11-aba-zoom.png` mostram a aba encostada na borda, e `12-gaveta-animando.png` mostra a gaveta visível 400 ms após o clique. A capability de eventos foi validada de ponta a ponta; sem ela a gaveta abriria invisível.

Referências consultadas:

- [Motion, Apple Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/motion). Movimento breve, preciso, opcional e cancelável.
- [Designing Fluid Interfaces, WWDC18 803](https://developer.apple.com/videos/play/wwdc2018/803). Resposta imediata, interrupção e continuidade espacial.
- [Animate with springs, WWDC23 10158](https://developer.apple.com/videos/play/wwdc2023/10158). Duração e bounce como parâmetros e preservação de velocidade na interrupção.
- [CSS Easing Functions Level 1, W3C](https://www.w3.org/TR/css-easing-1). Definição de `cubic-bezier()`.
- [corner-shape, MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/corner-shape) e [suporte no Can I Use](https://caniuse.com/mdn-css_properties_corner-shape). Superellipse fora do WKWebView atual.

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
- Material, blur e vibrancy ficam para GN-01C. O movimento leve de abrir e recolher entrou nesta entrega a pedido do mantenedor; a máquina completa com `opening` e `closing`, cancelamento, animação nativa de material, `Esc`, menus, "manter aberta" e persistência de posição continuam em GN-07.
- O comportamento de movimento reduzido está implementado em CSS, mas não foi exercitado nativamente porque a configuração de acessibilidade do sistema não foi alterada; a verificação fica para GN-07A.
- Ao recolher a gaveta pelo clique na aba, o foco não retorna automaticamente ao aplicativo anterior. O comportamento fica pendente para GN-07B.
- A medição rodou com outra instância do Git Notch em execução em outro checkout. A atribuição de auxiliares usou diferença de PIDs; um auxiliar de rede transitório observado na primeira medição pode pertencer ao conjunto compartilhado do WebKit.
- O critério de abertura visual offline herdado do GN-01A continua pendente e não é resolvido por este ticket.
