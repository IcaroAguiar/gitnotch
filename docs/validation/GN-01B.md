# Validação da aba e da gaveta nativas (GN-01B)

**Estado:** o candidato de QA da gaveta flutuante de 16/09/2026 tem verificações automatizadas, bundle e QA nativa limitada concluídos. Hover físico, foco externo, acessibilidade, monitores mistos, hit-testing exato e largura do conteúdo medida diretamente permanecem pendentes. As evidências históricas abaixo registram candidatos anteriores; não há aceite visual do usuário para este candidato.

Base do candidato atual: `f321d35976d432f2fc7408004d658d60fb4668ee`. Revisão histórica de código verificada: `1b7c8a64f8d18ed9c3d467ef1e8d3bf20dddf399`. O refinamento de encaixe partiu de `137de88d0d42fff635ed79f6f60692e06eff19eb`; o refinamento anterior de material partiu de `7de574964ada6509221e88e1805484ed329448a6`. Ambiente de build e QA: macOS 27.0, Apple Silicon, Node 22.23.2, pnpm 10.32.1 e Rust 1.98.1. O ambiente observado possui displays físicos de 2560 × 1440 e 1920 × 1080; esta entrega não alterou preferências de monitor ou acessibilidade.

## Candidato de QA: drawer flutuante em 16/09/2026

- A única `NSWindow` mantém a fita fechada de 28 × 112 encostada à direita e abre a mesma janela em 960 × 600, com quatro cantos de 20 px e lacuna de 24 px lógicos até a borda direita. A alça aberta é translúcida, mede 36 × 68 e reúne cabo grafite e chevron; o estado vazio continua sem dados Git artificiais.
- `pnpm check` passou com Biome, TypeScript e 7 testes Node. `pnpm rust:check` passou com rustfmt, Clippy em `-D warnings` e 35 testes Rust, inclusive margem em escala 2, área útil pequena/origem negativa, cantos abertos e corredor de hover. Logs: `artifacts/floating-drawer/logs/pnpm-check-candidate2-no-shadow.log` e `artifacts/floating-drawer/logs/rust-check-candidate2-no-shadow.log`.
- `pnpm exec tauri build --bundles app -- --locked` passou e produziu `src-tauri/target/release/bundle/macos/Git Notch.app`. O executável `Contents/MacOS/gitnotch` tem SHA-256 `b2b7a26469060b9a5f94fda8ac15f990fefc5b43f87d8837e6f5c3759670b630`. Log: `artifacts/floating-drawer/logs/tauri-bundle-app-candidate2-no-shadow.log`. `git diff --check` também passou.
- Na QA nativa do macOS 27.0, `artifacts/floating-drawer/final-light.png` e `final-dark.png` mostram o vidro limpo com quatro cantos, lacuna de 24 px e alça; `final-closed.png` mostra a fita restaurada. Os frames em 19 s (aberta) e 7 s (fechada) de `final-cycle.mp4` foram inspecionados; o MP4 derivado preserva os 20,044 s de timing do MOV. `Esc`, clique na alça, Recolher e múltiplas reaberturas foram observados.
- `artifacts/floating-drawer/final-geometry.jsonl` tem 808 amostras em 20 s para a única janela `2708`. Aberta: `1576,391,960,600`, direita `2536` em uma tela que termina em `2560` (lacuna 24). Fechada: `2532,635,28,112`, direita `2560`. O centro vertical foi `691`, ou `691,5` nos arredondamentos intermediários.
- O primeiro candidato ligou a sombra de `NSWindow` e apresentou borda e triângulos pretos nos cantos inferiores. O candidato acima mantém a sombra desabilitada; não há alegação de elevação por sombra.
- Ainda faltam hover com ponteiro físico, escala/DPI e monitores reais, acessibilidade, foco e hit-testing entre aplicativos, e a medição nativa direta da largura do conteúdo. A cobertura unitária desses contratos não substitui essas provas.

## Verificações automatizadas históricas

- `pnpm check` passou: Biome, TypeScript e 7 testes Node. Inclui o contrato da janela/capability e o descarte de snapshots atrasados, inclusive `Opening` da mesma geração após `Resting`. Log: `artifacts/premium-lateral/pnpm-check-final.log`.
- `pnpm build` passou e produziu assets estáticos locais em `dist`.
- `pnpm rust:check` passou: rustfmt, Clippy com `-D warnings` e 29 testes Rust. Os testes cobrem intenção/fase/geração, reversão durante abertura, guardas de interação, geometria limitada, cantos arredondados apenas à esquerda, conversão AppKit com origem negativa/escala 2 e raio de material. Log: `artifacts/premium-lateral/rust-check-final.log`.
- `pnpm exec tauri build --bundles app -- --locked` passou e produziu o aplicativo em `src-tauri/target/release/bundle/macos/Git Notch.app`. O executável do bundle final tem SHA-256 `0cd439586e673de45fcf02f5039ad9b5dea305ca8e041f5b7ffd81e5ae495f84`. Log: `artifacts/premium-lateral/desktop-bundle-app-final.log`.
- A tentativa de bundle padrão também produziu o executável e `.app`, mas falhou ao criar o DMG em `bundle_dmg.sh`. O DMG não foi recuperado nesta entrega; log: `artifacts/premium-lateral/desktop-bundle.log`.

## CI multiplataforma

A execução remota de `341e113` encontrou quatro avisos elevados a erro no Linux: duas constantes e uma função de animação exclusivas do macOS, além do handle sem uso no fallback. A correção posterior limita os símbolos de animação ao macOS e identifica o parâmetro opcional por plataforma. Ela não altera o comportamento macOS capturado acima. Os checks remotos dessa correção devem ser consultados na PR; não equivalem a aceite visual em Linux ou Windows.

## Contrato implementado

- Uma única forma nativa `notch` nasce em 28 × 112 encostada à direita e expande para até 960 × 600 lógicos; aberta, ela preserva 24 px lógicos da borda direita e limita a forma à área útil atual.
- Rust mantém `DrawerIntent` (`closed`, `preview`, `pinned`), `DrawerPhase` (`resting`, `opening`, `closing`) e geração monotônica.
- A animação macOS usa `NSAnimationContext`; o callback de conclusão confirma a fase somente se a geração ainda for atual. Não há temporizador fixo de conclusão no macOS.
- A checagem da geração e o início de foco/evento/frame ocorrem no main thread. O callback agenda a conclusão após liberar o mutex, evitando reentrância de duração zero.
- Material usa `NSGlassEffectView` `Regular` dentro de uma raiz de recorte do tamanho da janela. O vidro e seu host excedem 20 pontos somente em `closed/resting` e animam a zero ao abrir; o conteúdo original preserva a largura visível da janela. O fallback sólido restaura esse conteúdo sem reter ponteiro Objective-C cru.
- O frontend sincroniza preferências de movimento/transparência/esquema de cor, descarta snapshots atrasados e mantém a prévia aberta durante seleção ou captura real de ponteiro.

## Candidato anterior: refinamento líquido/material em 16/09/2026

Código do bundle verificado: `f0b2412bb75fc379a9d41e2f83a58df1f7cea659`. O commit seguinte apenas identifica este SHA na documentação.

- A abertura usa 380 ms com `cubic-bezier(.16,1,.3,1)`; o fechamento usa 240 ms com `cubic-bezier(.32,.72,0,1)`. O conteúdo abre após 120 ms e anima opacidade/deslocamento por 180 ms; movimento reduzido usa duração nativa zero e CSS de 1 ms.
- `NSGlassEffectView` continua no estilo `Regular`. `NSAppearanceNameAqua` é aplicado somente ao vidro durante a forma aberta ou fechando; a conclusão lê o estado atual e restaura `appearance = nil` em `closed/resting`. Não há definição de aparência em `NSWindow` nem no sistema; as subviews do vidro podem herdar Aqua.
- `pnpm check` passou (Biome, TypeScript e 7 testes Node); `pnpm rust:check` passou (rustfmt, Clippy com `-D warnings` e 29 testes Rust); o bundle `--bundles app -- --locked` passou. Logs: `artifacts/liquid-open/logs/pnpm-check.log`, `artifacts/liquid-open/logs/pnpm-rust-check.log` e `artifacts/liquid-open/logs/tauri-build-app.log`.
- O executável do bundle final em `src-tauri/target/release/bundle/macos/Git Notch.app` tem SHA-256 `3f6f6f30f7c67a7d14742a34d2ecaf015c75809de46093f2950c7b8c3cb4cecb`.
- `artifacts/liquid-open/final-light.png` e `final-dark.png` foram inspecionadas no harness sintético: a superfície perolada transmite cores/formas desfocadas e os textos permaneceram legíveis nos dois fundos. `artifacts/liquid-open/final-cycle.mov` (e a conversão sem retiming em `final-cycle.mp4`) registra 20 s compostos do app; clique na fita abriu, `Esc` fechou, a gaveta reabriu e Recolher fechou.
- `artifacts/liquid-open/final-geometry.jsonl` contém 810 amostras em 20 s para a janela `1764`: uma única janela, borda direita em 2.560, repouso em 28 × 112 e aberto em 960 × 600, centro 691 (691,5 nos arredondamentos intermediários).
- Permanecem pendentes hover físico, foco externo, acessibilidade, monitores mistos/desconexão, `innerWidth` direto e hit-testing/click-through exato nos cantos esquerdos. A QA descrita acima não equivale a aceite visual do usuário.

## Evidência nativa do segundo candidato

- O segundo `.app`, anterior à última correção de sincronização do material, foi executado no display primário do macOS 27.0. Seu executável tinha SHA-256 `0721742265c8f1638cf50bbc9a191b07d03e369a30315968cb0a54a245497af9`.
- Cinco ciclos pelos controles de fixar/recolher foram concluídos; `Esc` foi verificado uma vez separadamente. `artifacts/premium-lateral/native-candidate-cycles.mov` registra o roteiro; `artifacts/premium-lateral/native-candidate-demo.mp4` é o recorte de 13,5 s a 24 s, sem alteração de velocidade e sem expor o restante da tela.
- `artifacts/premium-lateral/native-candidate-geometry.jsonl` contém 1.070 amostras: a forma alternou 16 × 96 e 960 × 600, e `x + width` permaneceu 2.560 em todas as amostras. Esse registro prova geometria e colocação na borda direita; não mede FPS nem duração exata.
- Uma inspeção manual da região real da tela confirmou conteúdo de outro aplicativo visível e desfocado sob a forma. A PNG isolada da janela não contém esse fundo e não foi usada para inferir opacidade. Não há evidência publicada da tela inteira porque ela continha atividade pessoal.

## Smoke nativo anterior do bundle

- Antes do refinamento de encaixe, o bundle SHA-256 `0cd439586e673de45fcf02f5039ad9b5dea305ca8e041f5b7ffd81e5ae495f84` abriu, fixou, recolheu por `Esc`, recolheu pelo controle e abriu novamente.
- A janela observada tinha id 17.450, posição `x=1.600`, `y=391` e alvo 960 × 600 no display primário. `artifacts/premium-lateral/native-final-cycle.mov` e `artifacts/premium-lateral/native-final-open.png` registram o smoke. `artifacts/premium-lateral/native-final-demo.mp4` contém os primeiros sete segundos do vídeo, sem mudança de velocidade.
- O smoke não demonstra fluxo com dados reais, aceite visual do produto, hover com ponteiro físico, foco entre aplicativos, seleção/captura de ponteiro, cantos transparentes, preferências de acessibilidade ou segundo monitor.

## Evidência nativa já observada no primeiro candidato

Esta evidência refere-se ao primeiro `.app`, antes dos ajustes finais de ribbon, rail e conteúdo; não é aceite do candidato final.

- A forma abriu em 960 × 600 e os controles estavam expostos em acessibilidade.
- `artifacts/premium-lateral/first-build-transition.mov` registra 25 s de abrir/recolher. `artifacts/premium-lateral/first-build-geometry.jsonl` registrou borda direita em 2560 em todas as amostras, alvo 960 × 600, fechamento observado de aproximadamente 197 ms e abertura de aproximadamente 235 ms desde a primeira amostra detectada. As amostras CG são de aproximadamente 25 ms e não comprovam FPS de renderização nem duração exata.
- Uma captura JPEG da janela achatou alpha em cinza; o vídeo apresentou tom escuro normal. Isso não comprova nem reprova sozinho a transparência do material.

## Roteiro ainda pendente

A validação final precisa cobrir:

1. prévia por hover, sua tolerância de saída e reversão com ponteiro físico;
2. foco da fita e da gaveta entre aplicativos, além da navegação por teclado;
3. chegada do conteúdo durante todo o redimensionamento; a primeira captura do segundo candidato não mostrou ribbon residual, mas não cobre todos os frames;
4. clique exato nos cantos arredondados esquerdos e no aplicativo atrás;
5. seleção e captura real de ponteiro sem recolhimento automático;
6. interrupção abrir/recolher e preferências de movimento reduzido ou transparência reduzida;
7. display secundário, escalas distintas e desconexão de monitor;
8. uso sustentado de CPU/memória e orçamento de recursos.

## Limites conhecidos

- `NSView.hitTest` ou `pointer-events: none` não provam que um clique será entregue a outro aplicativo. Não há alegação de click-through externo sem esse roteiro nativo.
- Durante o morph de raio 12 ↔ 20, o polling usa raio 20 de forma conservadora apenas nos cantos esquerdos, para não capturar pixels que podem já estar transparentes. Na abertura, isso também pode rejeitar um canto esquerdo ainda visível; equivalência exata entre apresentação e hit-testing permanece pendente de prova nativa.
- O driver disponível não oferece movimento puro de mouse documentado, portanto timing de hover/reversão exige observação manual ou evidência nativa adequada.
- Material, foco, hit-testing, monitores mistos e acessibilidade não são comprovados por testes unitários, build ou screenshots isolados.

## Reprodução de build

```sh
# Rust do rust-toolchain.toml e pnpm devem estar no PATH.
source .local/use-rust.sh
export DEVELOPER_DIR=/Library/Developer/CommandLineTools
pnpm check
pnpm build
pnpm rust:check
pnpm exec tauri build --bundles app -- --locked
```

## Refinamento de encaixe da fita em 16/09/2026

O código do bundle verificado corresponde ao commit `473f917ac3e679ca1bdcb66cb7be6476f93fa74a`, sobre a base `137de88d0d42fff635ed79f6f60692e06eff19eb`. A fita mede 28 × 112 lógicos, mantém seu centro vertical e sua borda direita em `work.x + work.width`. A folha aberta continua na mesma `NSWindow`, também presa à direita e limitada a 960 × 600 sem avançar para outro monitor. A superfície e o hit-testing arredondam apenas os cantos esquerdos.

No macOS, uma raiz `NSView` do tamanho da janela recorta um `NSGlassEffectView` 20 pontos mais largo à direita. O `contentView` do vidro é um host de mesma largura, que contém o conteúdo Tauri na largura da janela com margem direita fixa de 20 pontos. A fita DOM fica acima de `drawer-content` em `closed`, `preview`, `pinned` e `closing`; ela usa um glyph Git discreto, rótulo dinâmico e `aria-pressed` somente quando fixada. O trilho SVG e a regra de container que ocultava a fita foram removidos.

- `pnpm check` passou. Log: `artifacts/ribbon-edge/logs/pnpm-check.log`.
- `pnpm build` passou. Log: `artifacts/ribbon-edge/logs/pnpm-build.log`.
- `pnpm rust:check` passou com 29 testes. Log: `artifacts/ribbon-edge/logs/pnpm-rust-check.log`.
- `pnpm exec tauri build --bundles app -- --locked` passou e produziu `src-tauri/target/release/bundle/macos/Git Notch.app`. O executável `Contents/MacOS/gitnotch` tem SHA-256 `7654e147a54b3a9b93a407b7874569a98ab0d476ea9127122b039600f3d48fc0`. Log: `artifacts/ribbon-edge/logs/tauri-build-app.log`.

O bundle acima foi aberto no macOS 27.0 no display primário. `artifacts/ribbon-edge/native-cycle.mov` registra 20 s em 960 × 600 e `artifacts/ribbon-edge/native-demo.mp4` contém o trecho entre 8 s e 20 s, sem aceleração, mostrando fechar e reabrir. Os frames 5 e 13 foram inspecionados nos estados aberto e recolhido. As capturas `artifacts/ribbon-edge/after-open.png` e `artifacts/ribbon-edge/after-closed.png` mostram a fita única, a ausência do trilho e a borda direita reta sem folga. A captura isolada preserva a área externa preta e não expõe o desktop.

`artifacts/ribbon-edge/after-geometry.jsonl` contém 718 amostras de 20 s para a janela `1087`. Há exatamente uma janela em todas as amostras. `x + width` permaneceu em `2560`; o centro permaneceu em `691` nos repousos e em `691` ou `691,5` durante arredondamentos intermediários. As dimensões observadas foram 28 × 112 recolhida e 960 × 600 aberta. A fita clicada abriu e recolheu; `Esc` e o controle Recolher também recolheram a gaveta.

Essa evidência confirma a silhueta, o encaixe à direita e as transições medidas. Ela não mede `innerWidth` diretamente nem comprova hover com ponteiro físico, foco entre aplicativos, acessibilidade, monitores mistos ou click-through/hit-testing exato dos cantos esquerdos.
