# Git Notch — especificação de implementação v0.1

**Data da pesquisa:** 9 de setembro de 2026  
**Estado:** proposta implementável; aprovação visual conceitual recebida, integração nativa e metas de desempenho ainda não validadas.  
**Produto:** utilitário local e somente de leitura para acompanhar diffs de múltiplos checkouts Git numa gaveta lateral.  
**Decisão principal:** Tauri 2 + React/TypeScript + Rust; Git CLI como fonte de estado; efeitos nativos com fallback.

## 1. Resultado esperado e limite do projeto

O usuário cadastra uma ou mais pastas-base, por exemplo `~/dev`. Git Notch descobre checkouts independentes e aninhados, acompanha atividade e apresenta sugestões de onde revisar mudanças. Uma aba discreta na borda da tela abre a gaveta sobre a ferramenta de desenvolvimento. O aplicativo não identifica agentes nem depende de T3 Code, Antigravity, Codex ou outro editor.

Fluxo principal: **pasta-base → atividade observada → repositório → grupo Git → arquivo → diff**.

A pasta `tetra` é um cenário de aceite, não uma regra de código. Selecionar uma pasta guarda-chuva filtra os checkouts contidos nela; não cria um repositório virtual nem mistura seus índices.

### Incluído na v0.1

- Descoberta em vários níveis, várias pastas-base, worktrees separados e erros isolados.
- Sugestões por atividade efetivamente observada, com distinção de alterações preexistentes.
- Staged, unstaged, não rastreados e conflitos identificados separadamente.
- Diff unificado somente de leitura, carregado sob demanda.
- Aba lateral, gaveta redimensionável, transparência configurável e movimento reduzido.
- Atualização por eventos com reconciliação; atualização manual.
- Pacotes para os sistemas efetivamente testados, com capacidade visual declarada.

### Não incluído

Editor, terminal, stage/unstage, commits, checkout de branch, histórico completo, comparação de branches, gerenciamento de agentes, atribuição a tarefas, integrações remotas, conta, banco de dados, sincronização, telemetria remota, atualizador automático ou Marketplace/App Store. Não capturar tela, injetar interface em IDEs ou solicitar permissão de Acessibilidade para descobrir o aplicativo ativo.

Os cartazes conceituais aprovados são referência de aparência, não uma obrigação de implementar todos os cartões, abas e números que aparecem neles. Na implementação, evitar repetição entre “sugestões” e “repositórios”: a mesma lista recebe indicação de atividade e filtros simples.

## 2. Pesquisa aplicada: o que reaproveitar

| Componente / referência | Constatação na pesquisa | Decisão para Git Notch |
|---|---|---|
| `tauri-apps/window-vibrancy` | Release 0.8.0, de 16/07/2026, inclui `apply_liquid_glass` e `LiquidGlassOptions`; possui código e exemplo Tauri. Licença MIT ou Apache-2.0. | Dependência preferida para o material nativo. Fixar versão no lockfile. [R1–R3] |
| `pierrecomputer/pierre`, pacote `@pierre/diffs` | Pacote consultado 1.4.1, Apache-2.0. Renderiza patches; possui React, modo unificado, temas e entradas de worker. | Usar apenas visualização de patches do Git, sem edição, anotações ou resolução. Carregamento local e sob demanda. [R4–R5] |
| `notify-rs/notify` | Documentação consultada para 8.2.0; backends nativos e polling. O crate `notify` declara CC0-1.0; crates auxiliares possuem suas próprias licenças. | Observar filesystem no Rust. Não confundir filtro de eventos com exclusão de watches nativos. [R6–R7] |
| `ahkohd/tauri-nspanel` | Conversão/construção de painéis AppKit, controle de foco e eventos; MIT ou Apache-2.0. | Referência e dependência condicional: adicionar somente se a janela Tauri não atender ao teste de foco/flutuação no macOS. Fixar commit, não branch móvel. [R8] |
| `TheBoredTeam/boring.notch` | Separação de abrir/fechar, tarefas canceláveis e proteção contra fechamento durante interações. GPL-3.0. | Referência de comportamento. Não copiar código para o projeto sem decidir explicitamente as obrigações de licença. [R9] |
| `hkandala/tauri-plugin-liquid-glass` | Implementa integração específica de Liquid Glass e variantes experimentais. | Não adotar em paralelo ao `window-vibrancy`; evitar duplicação e variantes não documentadas. [R10] |

**Reaproveitamento inteligente:** depender de peças pequenas que resolvem integração com o SO, eventos e desenho do diff; manter descoberta, política de leitura, identidade dos checkouts e experiência do produto no próprio projeto. Não fazer fork de um cliente Git completo.

As versões acima são candidatos pesquisados, não uma combinação já compilada. No primeiro marco, validar compatibilidade entre Tauri, WebView, `objc2`, `window-vibrancy` e renderizador; então registrar versões exatas, checksums e licenças transitivas. Não declarar “última versão” com base em README de uma branch de desenvolvimento.

## 3. Arquitetura e responsabilidades

Um projeto, um núcleo Rust, frontend estático empacotado. Não adicionar servidor HTTP ou processo Node à execução do aplicativo. Node/pnpm ficam restritos às ferramentas de desenvolvimento.

```text
React/TypeScript
  NotchEntry         — aba e indicador, sem importar o renderizador de diffs
  DrawerEntry        — árvore, busca local, visualizador, preferências
  DesktopBridge      — contratos tipados e tratamento de respostas obsoletas
             ⇅ IPC Tauri, comandos estreitos
Rust
  AppState           — estado autoritativo, revisões e sessões
  WorkspaceRegistry  — raízes, descoberta, identidade dos checkouts
  GitReader          — subprocessos permitidos, parsers, política de leitura
  WatchCoordinator   — watches, invalidações, debounce e reconciliação
  ActivityIndex      — atividade observada, novidades, ordenação
  DiffService        — limite de entrada/saída e cache em memória
  DesktopAdapter     — janelas, efeitos, monitores, foco, acessibilidade
  SettingsStore      — preferências locais com versão de schema
```

Não transformar esses módulos em serviços independentes ou microarquiteturas. Um repositório basta:

```text
git-notch/
  src/
    notch/  drawer/  diff/  settings/  shared/
  src-tauri/src/
    app_state.rs  commands.rs  discovery.rs  git/
    watch.rs  activity.rs  diff.rs  desktop/  settings.rs
  tests/fixtures/
  docs/
  package.json  pnpm-lock.yaml  Cargo.lock
```

React mantém estados de apresentação. Rust decide quais raízes e arquivos são acessíveis, executa o Git e arbitra revisões. Um resultado recebido pelo frontend nunca amplia acesso a caminhos.

Preferências em arquivo JSON versionado no diretório de configuração do aplicativo; gravação atômica por arquivo temporário e rename. Nenhum estado do Git Notch dentro dos repositórios. Diffs, conteúdo de arquivos e histórico de atividade ficam apenas em memória nesta versão.

## 4. Janela e comportamento de desktop

### 4.1 Forma nativa única

A implementação inicial aprovada usa uma única forma Tauri com o label `notch`.

- Ela inicia compacta como fita e a mesma WebView cresce para o painel lateral.
- O núcleo Rust, o bundle e o estado são únicos; não há segunda janela nem segundo WebContent para sincronizar.
- A forma nunca ocupa a tela inteira somente para observar ponteiro. A região fora de sua moldura continua pertencendo ao aplicativo atrás, sujeita à validação nativa de hit-testing.
- A forma escolhida evita a troca de foco e o custo de um segundo WebView. Não se conclui por isso que todos os casos de foco/hit-testing já foram aceitos.

### 4.2 Geometria inicial, em unidades lógicas

| Elemento | Implementação GN-01B |
|---|---|
| Fita visível | 28 × 112, encostada na borda direita e centralizada na área útil |
| Gaveta | até 960 × 600; a largura e altura são limitadas pela área útil, preservando 24 px de margem quando possível |
| Borda | direita nesta entrega |
| Posição vertical | centro da fita, limitado à área útil |
| Raio | fita 12 e painel 20 somente nos cantos esquerdos; lado direito reto |

Os tamanhos de design são convertidos pelo `scale_factor` para os pixels físicos da API Tauri. A área útil vem do monitor atual da forma, com fallback ao primário se ela ainda não estiver associada a um monitor. A borda direita da fita e da gaveta é `work.x + work.width`; a forma não avança para fora da área útil nem para outro monitor. No AppKit, o frame final é calculado a partir do frame corrente de `NSWindow` e da posição física corrente da própria forma; não depende da altura do monitor principal. Isso evita assumir uma origem global positiva ou escala única. Desconexão e monitores mistos ainda requerem teste nativo.

O notch é na **borda da tela**, não o recorte físico de câmera do Mac. Não acompanhar a janela da ADE.

### 4.3 Interação

Revisão de interação de 10/09/2026: a prévia por hover e a fixação por clique substituem a regra anterior de hover apenas decorativo. A fita recolhida continua sem roubar foco.

- O hover na fita abre uma prévia temporária após 120 ms de permanência. Clicar na fita fixa a gaveta; com a fita recolhida, o clique abre já fixado.
- A mesma fita DOM permanece visível em `closed`, `preview`, `pinned` e `closing`. Ela fica acima do conteúdo, usa glyph Git discreto e rótulo dinâmico; `aria-pressed` é verdadeiro somente quando a gaveta está fixada.
- A prévia não rouba foco. A gaveta fixada ignora a perda de foco e a saída do ponteiro.
- A prévia recolhe após tolerância de saída de 250 ms da região da fita e da gaveta; nova entrada cancela o fechamento. Seleção de texto e captura real de ponteiro suspendem o fechamento temporário. Menus e diálogos entram na mesma guarda quando existirem.
- Com a gaveta fixada, apenas um comando explícito recolhe: a própria fita, um controle de fixação, o botão Recolher ou `Esc`.
- O indicador fechado representa **repos com novidades observadas ainda não visualizadas**; tooltip explicita a unidade. Zero novidades não é um erro.
- Preferir ponto discreto a um badge vermelho permanente. Conflitos podem usar indicador distinto, acompanhado de texto ao abrir.
- Mudanças externas não abrem a gaveta nem roubam foco.
- Na prévia, clicar no conteúdo pode conceder foco à gaveta para interação; a fixação continua sendo uma intenção explícita. A fita recolhida não interrompe a digitação em outro aplicativo.
- `Esc` recolhe a gaveta fixada quando a página não o marcou como tratado; busca/menu/diálogo futuro deve tratá-lo antes. A prévia não captura o `Esc` digitado no aplicativo anterior.
- Clicar fora recolhe a prévia quando não houver seleção de texto, diálogo, menu ou operação de arraste em andamento. A gaveta fixada permanece até um comando explícito.
- Reabrir deve restaurar seleção, expansões e âncora de leitura antes de receber novos dados quando esses conteúdos forem implementados.
- Atalho global configurável, tray/menu bar e persistência de posição permanecem fora desta entrega.

A transparência visual não determina hit-testing. Validar cliques fora das janelas e nos cantos arredondados esquerdos em cada plataforma; os cantos direitos pertencem à superfície reta. `pointer-events:none` em HTML não equivale a passar o clique a outro aplicativo.

### 4.4 Matriz de capacidades

| Ambiente | Material desejado | Fallback e compromisso |
|---|---|---|
| macOS 26+ | Liquid Glass nativo, variante pública `Regular` | Vibrancy ou sólido se a integração falhar; informar modo efetivo. |
| macOS anterior compatível com o build | Vibrancy | Sólido. Não imitar Liquid Glass com shader. |
| Windows 11 | Acrylic quando disponível e com desempenho aceitável | Sólido. Mica não equivale a transparência sobre a IDE: usa o wallpaper. |
| Linux/X11 | Superfície consistente e sólida por padrão | Transparência somente onde comprovada; não exigir configurar compositor. |
| Linux/Wayland | Janela compacta convencional como baseline | Aba/posicionamento flutuante apenas se comprovados. Não prometer comportamento absoluto de posição e topo. |

`window-vibrancy` não fornece blur Linux. Há relato aberto no Tauri de limitações de posicionamento e always-on-top no Wayland; isso é evidência de risco, não prova de impossibilidade em todos os compositores. [R1, R11–R12]

**O suporte funcional multiplataforma é obrigatório; a paridade exata do notch não é.** Fullscreen/Spaces, versões mínimas e arquiteturas suportadas só entram na matriz publicada depois dos testes correspondentes.

## 5. Transparência: contrato visual e implementação

### 5.1 Três camadas, uma preferência

1. **Material nativo:** o SO desenha vidro/vibrancy/Acrylic atrás do conteúdo web.
2. **Preenchimento da interface:** fundo semitransparente, bordas e estados de seleção desenhados pelo frontend.
3. **Área do código:** superfície própria de alto contraste; texto, números e sinais sempre opacos.

A Apple reserva Liquid Glass principalmente à camada de controles e navegação; conteúdo denso não deve disputar legibilidade com o efeito. Para Git Notch, a moldura, cabeçalho e navegação recebem o vidro; o diff recebe um fundo de leitura. `Regular` é o ponto de partida, não `Clear` sobre código movimentado. [R13]

**Não usar `opacity` na janela inteira, `body` ou contêiner da aplicação como controle de transparência.** Isso desbota também letras e ícones. Uma animação curta de aparecimento pode usar opacidade; ela não é o ajuste permanente do usuário.

### 5.2 Preferências

```ts
type AppearanceSettings = {
  theme: 'system' | 'light' | 'dark';
  material: 'auto' | 'solid';
  transparency: number;       // inteiro 0..100; padrão proposto 60
  protectCodeReadability: boolean; // padrão true
  motion: 'system' | 'reduced';
};
```

O controle visível é **“Transparência do painel”**. Máximo significa deixar o material do SO aparecer mais, não transformar a janela em vidro totalmente claro nem reduzir a opacidade do texto. Os números não representam a mesma transmitância física entre sistemas.

Mapeamento inicial para o preenchimento, sujeito ao teste visual:

```ts
const t = Math.min(100, Math.max(0, transparency)) / 100;
const forceSolid = material === 'solid' || systemReduceTransparency || highContrast;
const panelFillAlpha = forceSolid ? 1 : 1 - t;
const codeFillAlpha = forceSolid || protectCodeReadability ? 1 : 1 - 0.08 * t;
```

Com padrão 60, o preenchimento do painel tem alfa 0,40 sobre o material nativo; o código permanece em 1. Se a proteção do código for desligada, a transparência dele continua limitada: alfa mínimo 0,92. São valores de partida, não recomendação oficial da Apple nem contraste já medido.

No fallback sem material nativo, a versão inicial usa fundo sólido: não mostrar simplesmente os pixels da IDE através das letras. Exibir “Transparência limitada neste ambiente”. O notch pode compartilhar a preferência, mas não herdar o fundo opaco do diff.

### 5.3 Integração nativa

A implementação GN-01B usa `NSGlassEffectView` público de AppKit, no estilo `Regular`, pelo adaptador Rust com `objc2`. Uma raiz `NSView` do tamanho da janela recorta o vidro, que excede 20 pontos internamente à direita. O `contentView` documentado do vidro é um host de mesma largura que contém o conteúdo existente da janela na largura visível, com margem direita fixa de 20 pontos. O adaptador não adiciona uma subview arbitrária ao vidro nem mantém ponteiro Objective-C cru em estado global. No candidato atual, `NSAppearanceNameAqua` fica restrito ao `NSGlassEffectView` enquanto a forma está aberta ou fechando; em `closed/resting`, `appearance = nil` restaura a herança do sistema. Essa sincronização não define aparência em `NSWindow` nem no sistema; as subviews do vidro podem herdar Aqua.

O adaptador aplica o material uma vez, recupera o vidro pela hierarquia raiz de recorte → vidro e restaura o conteúdo original extraído do host quando o efeito não estiver disponível ou quando `accessibilityDisplayShouldReduceTransparency` estiver ativo. O modo efetivo (`glass` ou `solid`) volta ao frontend. Mudanças em `prefers-reduced-transparency`, `prefers-reduced-motion` e esquema de cor pedem nova leitura do modo efetivo; o material e a redução de transparência ainda exigem validação nativa em execução.

Não há slider de transparência, persistência de aparência nem uso de `window-vibrancy` nesta entrega. O preenchimento CSS completa o material sem reduzir a opacidade permanente de texto. Não implementar blur do desktop com `backdrop-filter` no HTML nem capturas periódicas da tela. Operações AppKit ocorrem na main thread.

### 5.4 Acessibilidade e aceite visual

- Observar preferências do sistema em runtime; não presumir que todos os WebViews expõem todas as media queries.
- Reduzir transparência/alto contraste força fundo sólido, independentemente do slider.
- Reduzir movimento remove deslocamentos, elasticidade e morph; usar troca instantânea ou fade curto.
- Estados Git possuem letras/ícones e texto além de cor.
- Foco visível, zoom e seleção/cópia de código devem funcionar.
- Validar fundo da IDE claro, escuro e de alto contraste, janela atrás rolando e outro app com vídeo; não basta um wallpaper estático.
- O controle deve alterar superfícies, não a nitidez do texto.

Materiais Apple adaptam-se a configurações de acessibilidade; nossas camadas CSS e animações precisam acompanhar explicitamente. [R16]

## 6. Movimento e transições

Valores do candidato atual, ainda sujeitos ao aceite nativo:

| Transição | Duração / comportamento |
|---|---|
| Hover na fita | abertura após 120 ms de permanência |
| Saída da prévia | tolerância de 250 ms; nova entrada cancela |
| Abrir | 380 ms em `NSAnimationContext`, curva `cubic-bezier(.16,1,.3,1)` |
| Recolher | 240 ms em `NSAnimationContext`, curva `cubic-bezier(.32,.72,0,1)` |
| Movimento reduzido | duração nativa zero; CSS em 1 ms, sem atraso perceptível |

A transição nativa altera frame e raio da mesma forma. Cada intenção recebe geração e fase (`opening`, `closing` ou `resting`). A confirmação vem do callback de conclusão de `NSAnimationContext`, não de uma espera fixa; callbacks de geração anterior são descartados. O efeito é iniciado no main thread depois de checar a geração atual, para impedir que uma ação atrasada redimensione ou foque a forma depois de uma intenção nova.

O conteúdo usa apenas opacidade e deslocamento curto, com atraso de 120 ms e duração de 180 ms na abertura; no fechamento, sai antes do estreitamento. Não emitir `set_size`/`set_position` por frame, nem animar blur, tint ou refração no JavaScript. Os valores 280/200 ms e suas curvas anteriores permanecem somente nos registros históricos de GN-01B.

Estado da gaveta: as intenções são `closed`, `preview` e `pinned`; a fase visual é separada para permitir reversão. Seleção de texto e captura real de ponteiro suspendem o recolhimento automático da prévia. Menus, diálogos, busca e persistência de posição continuam fora da superfície vazia desta entrega. Durante o morph de raio, o hit-testing usa o maior raio como guarda conservadora somente nos cantos esquerdos; a equivalência com a camada apresentada pelo AppKit ainda precisa de prova nativa. O Boring Notch fornece referência de cancelamento e diferenciação entre abrir e fechar; não reutilizar seu código GPL por padrão. [R9]

## 7. Organização da gaveta

Uma coluna principal. Cabeçalho pequeno com nome, pasta/filtro atual, busca, atualizar e preferências. Sem dashboard de métricas, cartões promocionais ou três visões redundantes.

- Lista de checkouts com nome, caminho relativo não ambíguo, branch/estado e indicador de atividade.
- Abertura do repo revela grupos Git; abertura de arquivo revela diff unificado inline.
- Manter apenas um renderizador de diff pesado ativo na v0.1. Outras linhas/expansões podem permanecer abertas sem montar renderizadores ocultos.
- Cabeçalho do diff visível durante leitura: repo, arquivo e comparação.
- Staged: `HEAD → índice`; unstaged: `índice → arquivos locais`; novo: “Ainda não versionado”.
- Um arquivo nos dois grupos tem duas identidades de comparação e duas leituras independentes.
- Repos limpos ocultos por padrão; filtro para exibi-los.
- A ordem pode ser recalculada ao abrir a gaveta ou por ação explícita. Não mover as linhas enquanto o usuário navega.
- Em caminhos iguais, exibir também a pasta-base. Uma árvore virtual nunca apaga a identidade do checkout.

Se o arquivo selecionado ficar limpo: manter posição e exibir “Este arquivo não possui mais alterações nesta comparação”. Não selecionar o próximo arquivo automaticamente. Se o repo desaparecer: “Checkout indisponível”, preservando a navegação dos demais.

## 8. Descoberta e identidade dos repositórios

### 8.1 Algoritmo

1. O seletor nativo autoriza uma pasta-base. Resolver e registrar o caminho real.
2. Percorrer diretórios em fila, com cancelamento e resultados progressivos.
3. Encontrar `.git` como diretório ou arquivo; validar o candidato com Git.
4. Resolver o topo do checkout, gitdir privado e diretório Git comum por `rev-parse`.
5. Criar `repoId` opaco por checkout canônico e registrar seus aliases de pastas-base.
6. Continuar a procura de outros repos aninhados, sem descer em `.git`.

A identidade não é nome, branch, remoto nem common gitdir. Dois worktrees do mesmo repositório têm IDs distintos. Não converter todos os caminhos para minúsculas; case-sensitivity depende do filesystem. Preservar caminho de exibição separado da identidade.

Resolver worktree privado e comum pelo Git; índices e HEAD privados podem estar fora da pasta-base. Permitir leitura desses metadados porque pertencem ao checkout validado, não abrir acesso genérico à pasta externa. [R17]

### 8.2 Exclusões e limites

Padrões iniciais de descoberta: `.git`, `node_modules`, `.next`, `.nuxt`, `.turbo`, `.cache`, `.venv`, `venv`, `target`, `dist`, `build`, `coverage`. A lista é visível e ajustável; não excluir genericamente todo diretório oculto, `vendor` ou todo conteúdo do `.gitignore`.

Seleção explícita de uma pasta prevalece sobre exclusão de descoberta. Mostrar diretórios ignorados e permitir acrescentar uma pasta-base específica. Não seguir links simbólicos de diretórios na varredura inicial. Não tratar repositórios bare como checkouts revisáveis.

Limites iniciais propostos: profundidade 12, 100 mil diretórios visitados por varredura, resultados progressivos e cancelamento. Ao alcançar limite, mostrar “Descoberta parcial”, nunca “Todos os repositórios encontrados”. Permitir aumentar limite ou escolher uma raiz menor.

Exclusões de descoberta não são exclusões do status Git. Arquivos rastreados dentro de `dist` continuam pertencendo ao checkout e precisam ser detectados pelo plano de watches ou pela reconciliação.

### 8.3 Submódulos e links

Submódulo inicializado dentro da raiz pode aparecer como checkout próprio, identificado como tal. No pai, o gitlink é uma entrada diferente: mostrar alteração de ponteiro e/ou sujeira do submódulo sem duplicar um patch de conteúdo interno. Não inicializar, atualizar ou buscar submódulos. Não inicializados são apenas sinalizados.

Para arquivo rastreado que é symlink, ler o link com `read_link`, não o alvo externo. Mudanças de tipo devem ser exibidas como tais. Não seguir links simbólicos recém-introduzidos para ler arquivos fora do checkout.

## 9. GitReader: fidelidade, segurança e ausência de escritas

### 9.1 Fonte de verdade

Usar o Git instalado, localizar seu executável uma vez e manter o caminho validado. Não instalar Git automaticamente nem invocar login shell para encontrá-lo. Git ausente ou sem capacidade obrigatória produz instrução de erro.

O protocolo utiliza `status --porcelain=v2 -z`; não interpretar a saída humana do terminal. Git documenta o formato porcelain como interface estável para scripts e diferencia os dois estados de cada caminho. `git status` pode gravar atualização do índice por padrão; `--no-optional-locks` evita essa atualização opcional. [R18]

Os templates abaixo são composição de argumentos para subprocesso, **não autorização para aceitar comandos arbitrários**:

```text
git --no-optional-locks --no-lazy-fetch --literal-pathspecs
    -c core.fsmonitor=false -c protocol.allow=never
    -C <checkout> status --porcelain=v2 --branch -z
    --no-ahead-behind --untracked-files=all --find-renames=50%
```

```text
# Staged: omitir HEAD explícito para funcionar antes do primeiro commit.
git <guardas> -C <checkout> diff --cached
    --no-ext-diff --no-textconv --no-color --no-relative
    --src-prefix=a/ --dst-prefix=b/ --unified=3
    --diff-algorithm=myers --find-renames=50%
    -- <caminho-atual> [<caminho-anterior>]

# Unstaged: os mesmos argumentos, sem --cached.
```

A comparação dos testes deve usar estes mesmos parâmetros. Não prometer identidade byte a byte com qualquer configuração pessoal de terminal, driver ou algoritmo. Para uma renomeação, fornecer caminhos anterior e atual, preservando a associação indicada pelo status. [R19]

`--no-lazy-fetch` impede buscar automaticamente objetos ausentes. Desabilitar programas externos de diff/textconv é uma proteção diferente. Executar com lista de argumentos, paths literais e delimitador `--`; nunca concatenar uma linha de shell. [R19–R20]

### 9.2 Ambiente e subprocessos

- Limpar variáveis Git herdadas que redirecionem repositório, índice, worktree ou injetem configuração, comandos, traces e pagers. Restaurar somente o ambiente necessário e as guardas definidas pelo app.
- Definir `GIT_TERMINAL_PROMPT=0`, `GIT_OPTIONAL_LOCKS=0`, `GIT_NO_LAZY_FETCH=1`; impedir protocolos de transporte e não executar comandos de rede.
- Não sobrescrever globalmente a configuração do usuário. Guardas por processo, não `git config --global`.
- Desabilitar fsmonitor externo nas leituras; não criar hooks, daemon Git ou índice de busca no checkout.
- Não usar `safe.directory=*`; reportar a restrição do Git.
- Validar capacidades do Git no startup. O teste desta pesquisa executou Git 2.47.3; isso não valida automaticamente versões anteriores.
- Limitar stdout/stderr e tempo; drenar ambos simultaneamente. Ao cancelar/estourar limite, finalizar e colher o processo, sem órfãos.
- Erros esperados de comandos auxiliares, como chave de configuração inexistente, não devem ser confundidos com crash.

### 9.3 Filtros externos: limitação explícita

Um filtro `clean` ou `process` pode executar outro programa durante a leitura/normalização de conteúdo. `--no-ext-diff` e `--no-textconv` não desativam esses filtros. A documentação diferencia esses mecanismos, e um experimento sintético nesta pesquisa confirmou a execução de `clean` mesmo com as duas flags. [R21]

Política inicial:

1. Antes do primeiro status/diff de worktree, inspecionar configuração efetiva de filtros sem executá-los.
2. Se houver comandos externos configurados, verificar quais filtros se aplicam aos arquivos rastreados, por `ls-files -z` e `check-attr -z --stdin`, considerando atributos do índice e do worktree. Não bloquear todos os repos só porque Git LFS foi instalado globalmente. [R22]
3. Se um filtro externo efetivamente se aplicar, classificar o checkout como **leitura limitada por filtro externo** e não executar a consulta de worktree que possa acioná-lo. Mostrar nome do filtro e o motivo; demais repos continuam funcionando.
4. Invalidar a avaliação quando atributos/configurações relevantes mudarem. Filtros não são silenciosamente desabilitados para apresentar um diff alterado como se fosse fiel ao Git.
5. Suporte mais sofisticado a LFS/filtros é trabalho posterior, não uma permissão implícita para executar código.

Essa é uma política conservadora para repositórios locais de confiança. Não constitui sandbox contra um repositório malicioso que muda sua configuração entre preflight e execução. Isolamento adversarial completo está fora da v0.1. Arquivos de configuração incluídos pelo Git e atributos globais fazem parte dos inputs a acompanhar/revalidar.

### 9.4 Parsing e casos especiais

Tratar NUL como delimitador de caminhos; preservar bytes e campos de renomeação. No porcelain v2, tratar registros normais, renomeação/cópia, unmerged, não rastreados e cabeçalhos de branch. Não dividir caminhos por espaços, tabs ou newline. Usar IDs de arquivo opacos no IPC; nomes não-UTF-8 no Unix têm exibição escapada, não conversão com perda que mude o caminho acessado.

- **Unborn:** staged compara com árvore vazia implicitamente; `HEAD` não existe.
- **Detached:** mostrar commit curto e texto “detached HEAD”.
- **Binário:** respeitar classificação do Git/atributos quando disponível; não tentar renderizar bytes como código.
- **Não rastreado:** conteúdo lido com limite e mostrado como novo, sem `git add` e sem arquivo temporário no repo.
- **Grande/encoding não suportado:** mensagem específica e tamanho; não truncar silenciosamente o patch.
- **Conflito:** grupo explícito. Nesta versão não oferecer comparação ordinária enganosa nem resolução; identificar entradas não resolvidas.
- **Permissão/remoção/lock temporário:** erro e retry por repo. O estado anterior pode continuar visível com aviso de desatualizado.

## 10. Monitoramento e atividade: sem recalcular tudo

### 10.1 Plano de watches

Construir watches a partir de raízes autorizadas, diretórios descobertos e metadados Git reais. Observar diretórios para tolerar editores que salvam por rename/troca de inode. Não vigiar um arquivo isoladamente e perder a nova versão após atomic save.

macOS, Windows e Linux têm backends e limites distintos; `notify` oferece eventos e polling, mas sua documentação registra perdas, diferenças entre editores, limites de watches e problemas em filesystems remotos. Eventos são invalidações, não a fonte de verdade. [R6–R7]

No Linux, ignorar `node_modules` no callback não impede o consumo de watches recursivos nesse diretório. Usar plano que realmente deixe de registrar as subárvores excluídas, com watches não recursivos quando necessário; incluir diretórios de arquivos rastreados mesmo quando o nome estiver na lista de exclusão de descoberta. Reagir a criação/remoção de diretórios atualizando o plano.

Preferir fontes locais. WSL em mount cruzado, NFS, SMB e limites do kernel podem exigir polling; sinalizar modo degradado. Não alterar sysctl ou configurações da máquina automaticamente.

### 10.2 Agendamento

```text
evento → normalizar caminho → mapear checkout(s)
       → marcar dirty + acumular caminhos candidatos
       → debounce por checkout (250 ms; espera máxima 1 s)
       → consulta Git limitada
       → snapshot novo e revisão de atividade
       → atualizar indicador/lista
       → invalidar diff se a comparação visível depender da mudança
```

Limites iniciais: dois subprocessos Git simultâneos no app e um job ativo por checkout. Um checkout que recebe mais eventos fica pendente para uma nova leitura; não dispara jobs sem limite. Priorizar o diff visível sem impedir o progresso dos demais.

Fila limitada com coalescência por repo. Ao perder eventos ou atingir capacidade, marcar reconciliação necessária. Usar backoff para falhas repetidas; ações manuais podem solicitar nova tentativa.

Reconciliar status de forma escalonada, aproximadamente a cada 30 s; redescoberta leve por eventos de diretório e varredura periódica mais espaçada, inicialmente 5 min. Foco, retorno de suspensão e atualização manual também reconciliam. Não consultar todos os repos no mesmo instante e não sobrepor varreduras.

Aba fechada: watchers e metadados continuam; renderização e patches param. Não rodar diff completo de todos os arquivos em background.

### 10.3 O que significa “novo”

Três estados distintos:

- **Alterações preexistentes:** vistas no snapshot inicial. Não recebem data fictícia de criação.
- **Atividade observada:** inputs de um arquivo/índice mudaram desde o snapshot anterior.
- **Diff atualizado:** o patch efetivamente consultado mudou em relação ao patch anteriormente visualizado.

Um arquivo `M` que recebe mais código continua `M`: comparar apenas nomes/status não detecta a novidade. Usar fingerprints dos inputs relevantes: OIDs/modes do índice/HEAD e conteúdo dos arquivos candidatos, com limites e coalescência. Metadata-only touch não deve afirmar mudança de código.

Uma mudança de bytes pode ser neutralizada pela normalização do Git. Enquanto não houver patch confirmado, usar a linguagem “atividade detectada”, não “diff diferente”. Para o arquivo em leitura, confirmar pelo hash do patch normalizado.

Não calcular hashes de todos os arquivos a cada evento. Capturar baseline limitado de arquivos candidatos já alterados e atualizar os afetados; arquivos acima do limite recebem estado “conteúdo não verificado”. A reconciliação recupera estados Git, mas não reconstrói o horário exato de mudanças ocorridas com o app fechado.

`lastSeen` da sessão avança para a revisão realmente apresentada. Se novas mudanças chegam durante a leitura, elas continuam não vistas. Não afirmar que uma revisão foi aprovada, testada ou atribuída a um agente.

A ordenação usa atividade observada recente, depois repos alterados preexistentes e caminho estável. Não precisa LLM nem API externa.

## 11. Concorrência, cache e preservação de leitura

Identidade de uma comparação: `(workspaceEpoch, repoId, comparison, fileId)`; requisição também carrega `requestId` e revisão dos inputs.

A resposta só é aplicada se a época da pasta, seleção atual e token da requisição ainda coincidirem. Cancelamento reduz custo, mas validação da resposta é a proteção de correção. Ao trocar de pasta-base, incrementar a época e invalidar jobs/seleções fora do novo escopo.

Git e filesystem não oferecem uma transação única enquanto agentes escrevem. Verificar revisão/fingerprint antes e depois da leitura; se mudou, descartar, reagendar com limite e mostrar “Atualizando”. Não apresentar consistência instantânea fictícia em atividade contínua.

Cache em memória por comparação + revisão, com LRU limitada. Não reusar diff apenas porque o path/status não mudou. Configurações de atributos, mudanças de HEAD/índice e alterações de modo também invalidam entradas relevantes.

Preservar scroll por arquivo/grupo e uma âncora de linha/contexto, não apenas `scrollTop`. Aplicar atualização automática quando a âncora puder ser mantida; quando isso deslocaria a leitura, manter o snapshot e mostrar “Há uma versão mais recente — atualizar”. Pausar leitura congela somente o diff, não o monitoramento.

Ao retornar a um arquivo, apresentar primeiro o cache identificado como atual/desatualizado; nunca esconder a situação do cache. Se o renderizador virtualizado não atualizar ao mudar o patch, tratar como falha de integração; adicionar teste explícito para esse cenário. Houve issue já encerrada no projeto Pierre sobre atualização com virtualização, usada aqui como cenário de regressão e não como alegação de bug atual. [R23]

## 12. Contratos de IPC e modelo de dados

Modelo conceitual; não é uma biblioteca publicada:

```ts
type Comparison = 'staged' | 'unstaged' | 'untracked';
type RepoHealth = 'ready' | 'unavailable' | 'limited' | 'stale';

type RepoSummary = {
  id: string;
  rootId: string;
  relativePath: string;
  name: string;
  branch: { kind: 'branch' | 'detached' | 'unborn'; label: string };
  health: RepoHealth;
  revision: number;
  activity: {
    kind: 'baseline' | 'observed' | 'none';
    observedAt?: string;
    unseen: boolean;
  };
  counts: { staged: number; unstaged: number; untracked: number; conflicts: number };
  incomplete: boolean;
};

type FileEntry = {
  id: string;             // opaco; Rust mantém o caminho real/bytes
  repoId: string;
  comparison: Comparison;
  displayPath: string;
  previousDisplayPath?: string;
  status: 'added' | 'modified' | 'deleted' | 'renamed' | 'typechanged';
  inputRevision: number;
};

type DiffRequest = {
  requestId: string;
  workspaceEpoch: number;
  repoId: string;
  fileId: string;
  comparison: Comparison;
  expectedRevision: number;
};

type DiffResult = {
  requestId: string;
  workspaceEpoch: number;
  repoId: string;
  fileId: string;
  comparison: Comparison;
  revision: number;
  state: 'text' | 'binary' | 'too-large' | 'clean' | 'limited' | 'error';
  patch?: string;
  displayContent?: string; // apenas para arquivo novo, nunca path arbitrário
  reason?: string;
};
```

Conflitos possuem registro próprio e não são forçados para `FileEntry` de comparação ordinária. Uma implementação final pode usar união discriminada para tornar impossível enviar patch num estado binário ou erro.

Comandos autorizados: `select_root`, `remove_root`, `list_repositories`, `refresh_repository`, `get_file_changes`, `get_diff`, `set_seen_revision`, `get_settings`, `update_settings`, `toggle_drawer`, `get_desktop_capabilities`. A seleção inicial é feita pelo diálogo autorizado, não por caminho arbitrário vindo do webview.

Eventos: `discovery_progress`, `repository_updated`, `repository_unavailable`, `comparison_invalidated`, `desktop_capabilities_changed`. Coalescer eventos de lista; não transmitir o conteúdo de patches pelo canal de atividade.

Cada comando valida IDs, comparação permitida e limites no Rust. O notch recebe apenas indicadores e ação de abrir/recolher. O drawer recebe os comandos de leitura e preferências. Não expor `run_git(args)`, `exec(command)` ou `read_file(path)` genéricos.

## 13. Renderização de diffs

`@pierre/diffs` renderiza o patch retornado pelo Git. Não usar seu algoritmo próprio como fonte de comparação dos arquivos rastreados: o objetivo é aproveitar apresentação, não substituir semântica Git. [R4]

- Modo unificado, tema claro/escuro, números de linhas, sinais `+`/`−` e cabeçalhos de hunk.
- Import dinâmico ao abrir o primeiro arquivo; não incluir o pacote na entrada do notch.
- Iniciar com poucas linguagens utilizadas: TypeScript/JavaScript, JSON, Go, C#, SQL, YAML, Markdown; todas empacotadas localmente.
- Linguagem desconhecida vira texto simples. Worker opcional local depois de testar entradas oficiais; nenhuma chamada a CDN.
- Uma instância pesada ativa por vez; cache de dados separado de árvore DOM.
- Quebra de linha longa configurável localmente; rolagem horizontal por padrão, sem truncar código silenciosamente.
- Escapar conteúdo como texto. Arquivos Markdown/HTML não viram página executável e links presentes no código não são abertos automaticamente.
- Patch com encoding não representável produz limitação explícita, não texto substituído sem aviso.
- Fonte monospace local do SO. Não baixar fontes nem distribuir fontes proprietárias sem permissão.

Não adicionar Monaco/VS Code embedded na v0.1. Não habilitar modo de edição, seleção para stage, resolução ou comentários só porque o componente oferece essas funções.

## 14. Limites operacionais propostos

Estes valores são orçamentos iniciais configurados centralmente, não resultados de benchmark:

| Recurso | Limite inicial |
|---|---|
| Arquivo para leitura textual | 4 MiB |
| Patch renderizado | 1 MiB ou 20 mil linhas, o primeiro limite |
| Linha muito longa | Acima de 16 KiB, desabilitar highlight da linha/arquivo e usar texto simples |
| Saída de status | 8 MiB, com estado parcial explícito se exceder |
| Entradas na lista | Virtualizar acima de 200; limite de retorno 50 mil com aviso de incompletude |
| Cache de patches | Até 16 MiB e 20 entradas, prevalece o primeiro limite |
| Paralelismo Git | 2 global; 1 job por checkout |
| Timeout inicial | Status 5 s; diff 3 s; descoberta por jobs canceláveis |
| Erros repetidos | Backoff por repo, mantendo atualização manual |

Ao exceder o patch, descartar a resposta textual incompleta e mostrar limite. Não entregar meia estrutura para o parser nem manter stdout ilimitado esperando o Git terminar. O status parcial nunca pode aparecer como repo limpo.

Métricas locais: duração de descoberta/status/diff, latência evento→lista, tamanho de saída, cancelamentos, falhas, número de watches, cache e backlog. Logs rotativos pequenos, sem conteúdo de arquivos, patches, credenciais ou comandos de filtros. Preferir IDs a paths absolutos; diagnóstico detalhado só por exportação deliberada.

## 15. Segurança do aplicativo

Capacidades do Tauri por janela, CSP restritiva e autorização no Rust são camadas complementares. A documentação alerta que comandos customizados precisam de configuração própria para restringir seu uso: não presumir que qualquer comando `invoke_handler` já está restrito porque existe um arquivo de capabilities. [R24]

- Permissões mínimas: diálogo de pastas, preferências, operações de janela e comandos específicos.
- CSP somente recursos locais e canais necessários do IPC; worker local explicitamente autorizado se adotado.
- Nenhum conteúdo remoto, navegação remota dentro do WebView, plugin HTTP ou runtime de shell acessível ao frontend.
- Validar canonicalização e autorizações de raiz; tratar symlink race de forma defensiva. Threat model inicial é ambiente local confiável, não sandbox hostil.
- Não montar drives nem ler pastas não escolhidas; exceção estreita somente para os metadados Git resolvidos do checkout e configs/atributos que o próprio Git utiliza.
- Preferências próprias podem ser gravadas; arquivos, índice, refs, branches e configurações dos repositórios não.
- Alteração de pasta remove observadores e cache que deixaram de ser autorizados.

Não afirmar que o app “não faz nenhuma escrita no disco”: ele grava preferências/logs próprios. A garantia é não modificar o estado dos repositórios por iniciativa do Git Notch dentro do escopo suportado.

## 16. Plano de validação

### 16.1 Fixtures Git

Criar árvore descartável semelhante à Tetra, sem depender dos projetos pessoais. Cobrir:

- Checkouts independentes em três profundidades; mesmos nomes e raízes sobrepostas.
- Staged/unstaged no mesmo arquivo; novo, removido, rename puro/com edição, mudança de modo/tipo.
- Unborn, detached, conflito, submódulo inicializado/não inicializado e worktrees com metadados externos.
- Binário, arquivo acima do limite, linha gigante, Unicode, espaços, quebras de linha no nome e pathspec literal.
- Links simbólicos; nomes não-UTF-8 onde o SO permitir; diretório excluído contendo arquivo rastreado.
- Atomic save, criação/remoção de repo, falha de permissão, burst de eventos e reconciliação após evento perdido.
- Filtro `clean`/`process` sintético que grava um marcador: o app deve barrar antes da execução. Fsmonitor externo também não pode executar.
- Clone parcial com objeto ausente: falhar localmente, sem rede/busca automática.

Comparar saídas com comandos diretos usando a mesma política. Para provar ausência de escrita, interromper outros escritores e comparar bytes/hash de arquivos, índice, refs/HEAD e configurações antes/depois. Não usar atime como indicador de mutação do repositório.

### 16.2 Concorrência e experiência

Trocar rapidamente repo, arquivo e grupo durante respostas lentas; nenhuma resposta antiga pode aparecer na seleção nova. Remover raiz com jobs em andamento. Alterar arquivo sem mudar status `M`. Limpar arquivo selecionado. Nova revisão enquanto a leitura está pausada. Abrir/fechar repetidamente durante animação. Alterar patch com renderizador virtualizado já montado.

### 16.3 Desktop real

Testar **build de produção**, não só browser/Vite:

- Vidro/fallback com IDE atrás, nos dois temas e com transparência variando.
- Texto nítido, seleção, busca e atalhos; movimento/transparência reduzidos.
- Foco não roubado ao editar externamente; cliques fora da aba chegam à IDE.
- Dois monitores, DPI diferente, monitor desconectado, suspensão e reconexão.
- Espaços/fullscreen no macOS; geometria e resize Windows; X11 e Wayland separados.
- Iniciar aplicativo empacotado fora do ambiente de desenvolvimento e com Git ausente.
- 100 ciclos de material/abrir/fechar não acumulam views, listeners ou memória de forma contínua.

### 16.4 Orçamentos de desempenho

Fixture inicial proposta: 20 checkouts, 100 mil arquivos rastreados somados, até 2 mil entradas alteradas, dois checkouts recebendo 5 eventos/s cada; registrar também o resultado na árvore real do usuário.

| Métrica | Meta inicial a validar |
|---|---|
| Aplicativo interativo a frio | Até 2 s, sem bloquear pela descoberta completa |
| Conteúdo útil da gaveta quente | Até 150 ms com snapshot em memória |
| Descoberta inicial da fixture | Até 5 s; mostrar resultados progressivos |
| Evento → indicação, carga normal | p95 até 1 s; burst declarado até 2 s |
| Diff textual até 200 KiB | p95 até 500 ms |
| CPU recolhido, sem mudanças | Média ≤0,5% de um núcleo durante 5 min; registrar picos de reconciliação |
| Memória residente atribuída ao app | Orçamento inicial 150 MiB recolhido / 250 MiB com diff; metodologia inclui auxiliares e explicita compartilhamento |
| Movimento | Sem tarefas JS longas recorrentes; medir frames no monitor de referência |
| Pacote | Registrar comprimido/instalado por SO, separando runtime WebView/Git; não usar tamanho do Rust binário como tamanho total |

São metas, não promessa de resultados. Não fixar teto único de distribuição comparando `.app`, instalador WebView2 offline e AppImage como se contivessem os mesmos componentes. Se exceder memória, primeiro reduzir WebViews montados, renderizadores e linguagens; não trocar de stack sem diagnóstico.

## 17. Distribuição e manutenção

“Portátil” significa código comum e pacotes por plataforma, não um executável universal sem pré-requisitos.

- macOS: pacote `.app` e distribuição local; assinatura/notarização para uma entrega confiável fora da máquina de desenvolvimento. Apple Silicon primeiro na validação real; Intel somente com build/teste correspondente.
- Windows: pacote Tauri que documenta a dependência WebView2 e o Git. Não chamar instalador offline com runtime embutido de “mesmo tamanho” do executável.
- Linux: escolher um formato principal conforme ambiente-alvo, explicitar runtime/compositor e validar a janela fallback. Não anunciar Linux genericamente a partir de um único teste X11.
- Atualização inicialmente manual. Não adicionar serviço de updates, conta, servidor ou pipeline de auto-publicação no escopo pessoal.

CI inicial: format/lint/typecheck, testes Rust/TS, fixtures Git e builds da matriz disponível. O agente não pode marcar teste macOS/Windows como aprovado com base apenas em execução Linux ou screenshots geradas.

Guardar licença/NOTICE de dependências no pacote. Fixar dependências e commits; revisar atualizações deliberadamente. Não executar scripts remotos de instalação ou skills de terceiros apenas porque um README os recomenda.

## 18. Marcos e critérios de parada

**M0 — risco nativo e vertical slice.** Aba, gaveta e um patch fixo legível em build real; variar transparência, validar foco, hit-testing e fallback. Em paralelo, descobrir dois repos e consultar um diff real por Git seguro. Decide estratégia de janelas e compatibilidade das dependências.

**M1 — utilitário utilizável.** Descoberta da Tetra, identidades, status, diffs e casos comuns; UI restaurável; nenhuma escrita nas fixtures. Não aprimorar detalhes estéticos antes dessa prova.

**M2 — atividade confiável.** Watch plan, debounce, reconciliação, novidades reais, filas e respostas obsoletas; experiência de leitura estável.

**M3 — acabamento e distribuição limitada.** Liquid Glass/fallbacks, transparência/acessibilidade, microinterações, medição e pacotes nos sistemas testados.

Estimativa de planejamento para um desenvolvedor familiarizado com as ferramentas: M0 1–2 dias; M1/M2 mais 3–5; M3 mais 3–5. Total aproximado 7–12 dias de engenharia, não prazo garantido. Setup de assinatura, bugs de compositor e ambiente de testes podem acrescentar esforço. A fidelidade do morph visual é o primeiro item a simplificar, não a segurança nem a separação entre checkouts.

**Concluído:** o usuário abre o Git Notch em cima da ADE, encontra onde houve atividade, revisa uma alteração que atravessa repos e recolhe sem perder foco ou posição; a navegação é somente leitura e os custos medidos ficam aceitáveis. Não expandir para cliente Git completo após esse ponto.

## 19. Evidência produzida nesta pesquisa

Foi executada uma prova isolada, em Linux com Git 2.47.3, usando somente repositórios temporários. Sete verificações passaram: identificação unborn; diff staged antes do primeiro commit; separação staged/unstaged do mesmo arquivo; preservação de bytes de index/HEAD/config nas consultas testadas; worktree com `.git` arquivo e gitdir distinto; demonstração de execução do filtro `clean` apesar de flags de diff; identificação do filtro com `check-attr` sem executá-lo.

A verificação de filtro demonstra **um risco a bloquear**, não que já existe um executor seguro implementado. Não foram testados aqui a pasta real Tetra, o aplicativo, watchers, transparência nativa, animações, builds macOS/Windows nem os orçamentos de desempenho. Resultado bruto: `research/git-readonly-probe-results.json` no pacote de entrega.

## 20. Fontes primárias

Consultadas em 09/09/2026. Código em branch principal pode mudar; os pins explícitos abaixo preservam os trechos onde usados.

- **R1:** Tauri, window-vibrancy — https://github.com/tauri-apps/window-vibrancy
- **R2:** Release 0.8.0 — https://github.com/tauri-apps/window-vibrancy/releases/tag/window-vibrancy-v0.8.0
- **R3:** Implementação Liquid Glass e exemplo no tag 0.8.0 — https://github.com/tauri-apps/window-vibrancy/blob/window-vibrancy-v0.8.0/src/macos/liquid_glass.rs ; https://github.com/tauri-apps/window-vibrancy/blob/window-vibrancy-v0.8.0/examples/tauri/src-tauri/src/main.rs
- **R4:** Pierre, pacote de diffs — https://github.com/pierrecomputer/pierre/tree/main/packages/diffs ; https://diffs.com
- **R5:** Manifesto do pacote — https://github.com/pierrecomputer/pierre/blob/main/packages/diffs/package.json
- **R6:** Notify, documentação 8.2.0 — https://docs.rs/notify/8.2.0/notify/
- **R7:** Notify, repositório e licenças — https://github.com/notify-rs/notify
- **R8:** NSPanel para Tauri — https://github.com/ahkohd/tauri-nspanel
- **R9:** Boring Notch e trecho inspecionado — https://github.com/TheBoredTeam/boring.notch ; https://github.com/TheBoredTeam/boring.notch/blob/99900bf630a3d3e97fae079df2175993318d51f7/boringNotch/ContentView.swift
- **R10:** Plugin Liquid Glass alternativo — https://github.com/hkandala/tauri-plugin-liquid-glass
- **R11:** Microsoft, Mica — https://learn.microsoft.com/en-us/windows/apps/design/style/mica
- **R12:** Tauri, relato de limitações Wayland — https://github.com/tauri-apps/tauri/issues/14913
- **R13:** Apple, Human Interface Guidelines: Materials — https://developer.apple.com/design/human-interface-guidelines/materials
- **R14:** Apple, NSGlassEffectView.Style — https://developer.apple.com/documentation/appkit/nsglasseffectview/style-swift.enum
- **R15:** Tauri, configuração `macOSPrivateApi` — https://v2.tauri.app/reference/config/
- **R16:** Apple, Meet Liquid Glass, WWDC25 — https://developer.apple.com/videos/play/wwdc2025/219/
- **R17:** Git worktree — https://git-scm.com/docs/git-worktree
- **R18:** Git status — https://git-scm.com/docs/git-status
- **R19:** Git diff — https://git-scm.com/docs/git-diff
- **R20:** Git, opções globais e ambiente — https://git-scm.com/docs/git
- **R21:** Git attributes e filtros — https://git-scm.com/docs/gitattributes
- **R22:** Git check-attr — https://git-scm.com/docs/git-check-attr
- **R23:** Pierre, regressão encerrada sobre atualização/virtualização — https://github.com/pierrecomputer/pierre/issues/876
- **R24:** Tauri, security capabilities — https://v2.tauri.app/security/capabilities/
