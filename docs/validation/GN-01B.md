# Validação da aba e da gaveta nativas (GN-01B)

**Estado:** implementação, verificações automatizadas, validação limitada do segundo candidato e smoke do binário final concluídos; aceite visual do produto e cenários sem automação permanecem pendentes.

Base verificada: `848ec9670ca45fca489fb9847bf00616bad2fdca`, com as alterações deste ticket ainda não commitadas. Ambiente de build: macOS 27.0, Apple Silicon, Node 22.23.2, pnpm 10.32.1 e Rust 1.98.1. O ambiente observado possui displays físicos de 2560 × 1440 e 1920 × 1080; esta entrega não alterou preferências de monitor ou acessibilidade.

## Verificações automatizadas

- `pnpm check` passou: Biome, TypeScript e 7 testes Node. Inclui o contrato da janela/capability e o descarte de snapshots atrasados, inclusive `Opening` da mesma geração após `Resting`. Log: `artifacts/premium-lateral/pnpm-check-final.log`.
- `pnpm build` passou e produziu assets estáticos locais em `dist`.
- `pnpm rust:check` passou: rustfmt, Clippy com `-D warnings` e 29 testes Rust. Os testes cobrem intenção/fase/geração, reversão durante abertura, guardas de interação, geometria limitada, quatro cantos arredondados, conversão AppKit com origem negativa/escala 2 e raio de material. Log: `artifacts/premium-lateral/rust-check-final.log`.
- `pnpm exec tauri build --bundles app -- --locked` passou e produziu o aplicativo em `src-tauri/target/release/bundle/macos/Git Notch.app`. O executável do bundle final tem SHA-256 `0cd439586e673de45fcf02f5039ad9b5dea305ca8e041f5b7ffd81e5ae495f84`. Log: `artifacts/premium-lateral/desktop-bundle-app-final.log`.
- A tentativa de bundle padrão também produziu o executável e `.app`, mas falhou ao criar o DMG em `bundle_dmg.sh`. O DMG não foi recuperado nesta entrega; log: `artifacts/premium-lateral/desktop-bundle.log`.

## Contrato implementado

- Uma única forma nativa `notch` nasce em 16 × 96 e expande para até 960 × 600 lógicos, limitada pela área útil do monitor atual.
- Rust mantém `DrawerIntent` (`closed`, `preview`, `pinned`), `DrawerPhase` (`resting`, `opening`, `closing`) e geração monotônica.
- A animação macOS usa `NSAnimationContext`; o callback de conclusão confirma a fase somente se a geração ainda for atual. Não há temporizador fixo de conclusão no macOS.
- A checagem da geração e o início de foco/evento/frame ocorrem no main thread. O callback agenda a conclusão após liberar o mutex, evitando reentrância de duração zero.
- Material usa `NSGlassEffectView` `Regular` como content view, com fallback sólido quando a classe não existe ou o sistema pede redução de transparência. Nenhum ponteiro Objective-C cru é retido.
- O frontend sincroniza preferências de movimento/transparência/esquema de cor, descarta snapshots atrasados e mantém a prévia aberta durante seleção ou captura real de ponteiro.

## Evidência nativa do segundo candidato

- O segundo `.app`, anterior à última correção de sincronização do material, foi executado no display primário do macOS 27.0. Seu executável tinha SHA-256 `0721742265c8f1638cf50bbc9a191b07d03e369a30315968cb0a54a245497af9`.
- Cinco ciclos pelos controles de fixar/recolher foram concluídos; `Esc` foi verificado uma vez separadamente. `artifacts/premium-lateral/native-candidate-cycles.mov` registra o roteiro; `artifacts/premium-lateral/native-candidate-demo.mp4` é o recorte de 13,5 s a 24 s, sem alteração de velocidade e sem expor o restante da tela.
- `artifacts/premium-lateral/native-candidate-geometry.jsonl` contém 1.070 amostras: a forma alternou 16 × 96 e 960 × 600, e `x + width` permaneceu 2.560 em todas as amostras. Esse registro prova geometria e colocação na borda direita; não mede FPS nem duração exata.
- Uma inspeção manual da região real da tela confirmou conteúdo de outro aplicativo visível e desfocado sob a forma. A PNG isolada da janela não contém esse fundo e não foi usada para inferir opacidade. Não há evidência publicada da tela inteira porque ela continha atividade pessoal.

## Smoke nativo do bundle final

- Após a correção que lê o raio do material no main thread sob o mutex, o bundle SHA-256 `0cd439586e673de45fcf02f5039ad9b5dea305ca8e041f5b7ffd81e5ae495f84` abriu, fixou, recolheu por `Esc`, recolheu pelo controle e abriu novamente.
- A janela observada tinha id 17.450, posição `x=1.600`, `y=391` e alvo 960 × 600 no display primário. `artifacts/premium-lateral/native-final-cycle.mov` e `artifacts/premium-lateral/native-final-open.png` registram o smoke.
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
4. rail/notch escuro na borda direita e aceite visual contra a referência;
5. clique nas bordas arredondadas e no aplicativo atrás;
6. seleção e captura real de ponteiro sem recolhimento automático;
7. interrupção abrir/recolher e preferências de movimento reduzido ou transparência reduzida;
8. display secundário, escalas distintas e desconexão de monitor.

## Limites conhecidos

- `NSView.hitTest` ou `pointer-events: none` não provam que um clique será entregue a outro aplicativo. Não há alegação de click-through externo sem esse roteiro nativo.
- Durante o morph de raio 8 ↔ 20, o polling usa raio 20 de forma conservadora para não capturar pixels que podem já estar transparentes. Na abertura, isso também pode rejeitar um canto ainda visível; equivalência exata entre apresentação e hit-testing permanece pendente de prova nativa.
- O driver disponível não oferece movimento puro de mouse documentado, portanto timing de hover/reversão exige observação manual ou evidência nativa adequada.
- Material, foco, hit-testing, monitores mistos e acessibilidade não são comprovados por testes unitários, build ou screenshots isolados.

## Reprodução de build

```sh
source .local/use-rust.sh
export DEVELOPER_DIR=/Library/Developer/CommandLineTools
pnpm check
pnpm build
pnpm rust:check
pnpm exec tauri build --bundles app -- --locked
```
