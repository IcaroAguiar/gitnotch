# Decisões de implementação

## Aplicação mínima

O aplicativo começa com uma janela Tauri convencional e uma tela de desenvolvimento. A estratégia de notch e drawer será validada separadamente, conforme a especificação. A tela inicial não lê repositórios nem solicita acesso ao filesystem.

React e Vite produzem assets locais em dist. Rust incorpora esses assets no executável. Não há servidor HTTP, Node em produção, fontes remotas ou chamadas de API. A configuração CSP restringe conexões do frontend aos recursos locais e ao transporte IPC do Tauri. Os comandos estreitos de desktop registrados em GN-01B não concedem leitura arbitrária de caminhos.

O fluxo inicial usa builds pontuais. Nenhum servidor persistente de desenvolvimento é necessário. Um futuro fluxo com Vite persistente deve respeitar as regras de Portly do ambiente do mantenedor; não é pré-requisito de execução do aplicativo distribuído.

As ferramentas têm versões fixadas nos manifests e arquivos de toolchain. pnpm-lock.yaml e src-tauri/Cargo.lock registram as resoluções. A CI verifica builds nos três sistemas, mas build aprovado não comprova interação nativa ou suporte completo.

Referências consultadas:

- [Tauri com Vite](https://v2.tauri.app/start/frontend/vite/).
- [Configuração de frontend Tauri](https://v2.tauri.app/start/frontend/).
- [Permissões Tauri](https://v2.tauri.app/security/permissions/).

O ícone provisório tem fonte em `src-tauri/icons/source.svg`. Para regenerar, execute `pnpm exec tauri icon src-tauri/icons/source.svg --output .local/generated-icons` e copie somente icon.png, icon.ico e icon.icns para src-tauri/icons. Os demais formatos gerados não pertencem a este aplicativo desktop.

## Executor Git de leitura e política de segurança (GN-02)

O módulo `GitReader` implementa a fonte de verdade baseada no Git instalado no sistema operacional, sem invocar shell, login shell ou instalar executáveis. A localização do binário ocorre uma vez no PATH e valida que a versão seja igual ou superior a 2.22, garantindo suporte completo ao formato porcelain v2 e às flags de segurança.

Todas as chamadas Git utilizam subprocessos com listas explícitas de argumentos e guardas obrigatórias:
`--no-optional-locks`, `--no-lazy-fetch`, `--literal-pathspecs`, `-c core.fsmonitor=false`, `-c protocol.allow=never`. O ambiente herdado é limpo de variáveis `GIT_*` e `PAGER`, e são impostas variáveis determinísticas de segurança (`GIT_TERMINAL_PROMPT=0`, `GIT_OPTIONAL_LOCKS=0`, `GIT_NO_LAZY_FETCH=1`).

O ciclo de vida dos subprocessos protege contra esgotamento de recursos e deadlocks de pipe: stdout e stderr são drenados concorrentemente por threads dedicadas com teto estrito de bytes (20 MiB por padrão) e timeout (15 s por padrão). O estouro de limite ou tempo encerra o processo via sinal e colhe o processo (`wait`), sem deixar processos zumbis ou órfãos.

Filtros externos (`clean` ou `process`) podem executar código arbitrário durante leituras do worktree. Conforme a seção 9.3 da especificação, implementou-se preflight que inspeciona a configuração efetiva (`filter.*.clean` e `filter.*.process`) e cruza com os atributos dos arquivos rastreados via `ls-files -z` e `check-attr -z filter --stdin`. Quando um filtro externo se aplica a arquivos rastreados, o repositório é classificado como `LimitedByExternalFilter`, recusando consultas de worktree que acionem o filtro e preservando o isolamento do produto.

O parsing utiliza `status --porcelain=v2 --branch -z --no-ahead-behind --untracked-files=all --find-renames=50%`, tratando bytes e delimitadores NUL sem quebra por espaços ou tabs. Um mesmo arquivo com modificações staged e unstaged (`MM`) é categorizado em ambos os grupos. Repositórios `unborn` e `detached HEAD` são tratados explicitamente. Diffs de arquivos não rastreados são gerados por leitura direta delimitada, sem `git add` e sem criar arquivos temporários dentro do repositório.

## Aba e gaveta nativas (GN-01B)

A implementação usa **uma forma Tauri nativa** com o label `notch`, em vez de duas janelas/WebViews. Ela nasce como fita de 28 × 112 lógicos e, quando a intenção muda para `preview` ou `pinned`, cresce para até 960 × 600 lógicos. A forma continua ancorada na borda direita da área útil do monitor atual. Em uma área menor, ela reduz largura e altura sem ultrapassar a área útil, mantendo a borda direita encostada e a margem de segurança no lado esquerdo ou vertical quando necessária. A escolha reduz o custo e a ambiguidade de foco de uma segunda janela, mas ainda requer aceite nativo para comportamento real de foco e hit-testing.

A referência visual escolhida é a fita de borda do [Yoink](https://eternalstorms.at/yoink/mac/) e o acesso persistente do [SideNotes](https://www.apptorium.com/sidenotes). A decisão traduz somente a relação espacial: uma alça estreita presa à borda direita e uma superfície que cresce para a esquerda, preservando a junção reta. Não reaproveita código, assets ou comportamento desses produtos.

A área útil vem primeiro de `current_monitor` da forma e usa o monitor primário apenas como fallback. Os retângulos de Tauri usam pixels físicos. No AppKit, o adaptador deriva o frame-alvo da moldura atual de `NSWindow` e da posição externa atual da própria forma, em vez de converter pela altura de `NSScreen.mainScreen`. Isso preserva origem negativa e escala do monitor atual no cálculo; o teste unitário cobre uma origem negativa com escala 2. A geometria real em monitores mistos continua pendente de validação nativa.

O Rust é a fonte de verdade do estado da gaveta:

- `DrawerIntent`: `closed`, `preview` e `pinned`;
- `DrawerPhase`: `resting`, `opening` e `closing`;
- geração monotônica para cada intenção que muda a apresentação.

Uma intenção nova invalida a conclusão anterior. Fixar durante a abertura cria uma nova transição `pinned/opening`, então a conclusão da prévia não pode assentar o painel. O frontend aceita apenas snapshots de geração mais recente e, na mesma geração, recusa uma fase animada recebida depois de `resting`.

A checagem da geração acontece no main thread imediatamente antes de foco, evento e frame nativos. O mutex permanece retido somente enquanto o efeito é enfileirado; a conclusão de `NSAnimationContext` agenda a atualização de fase depois dessa região crítica. Isso evita o temporizador estimado que existia antes e rejeita callbacks obsoletos, sem afirmar que a continuidade visual de uma interrupção já foi aceita em hardware.

O hover abre após 120 ms de permanência e a prévia tolera 250 ms de saída. Abrir dura 280 ms e recolher 200 ms; reduzir movimento entrega duração nativa zero e remove os atrasos CSS. `InteractionGuards` suspende o recolhimento automático durante seleção e durante captura real de ponteiro. O frontend espelha seleção, `gotpointercapture`/`lostpointercapture` e limpa a guarda em `pointerup`, cancelamento ou perda de foco; ele não captura artificialmente ponteiros de botões.

No macOS, o material usa `NSGlassEffectView` público no estilo `Regular` dentro de uma raiz `NSView` com o frame da janela e `clipsToBounds` ativo. O vidro e seu host de conteúdo excedem 20 pontos somente à direita, mas o conteúdo Tauri mantém o frame da janela e a margem direita fixa dentro do host. Assim, os cantos direitos do vidro ficam fora do recorte e a borda direita da forma permanece reta. O adaptador nunca guarda ponteiro Objective-C cru: localiza vidro por raiz de recorte e restaura o conteúdo original do host ao cair no fallback sólido. A leitura do raio ocorre no main thread sob o mutex imediatamente antes de instalar o efeito. A redução de transparência força sólido. O preenchimento CSS mantém uma única fita DOM sobre o conteúdo, reserva sua largura mais 20 px de respiro e arredonda apenas os cantos esquerdos. O estado vazio é intencional e não inventa repositórios, arquivos ou diffs.

A capability declarada para a forma continua limitada a escutar o evento de estado. Os comandos registrados são `toggle_drawer`, `collapse_drawer`, `set_drawer_interaction`, `get_desktop_capabilities` e `refresh_desktop_appearance`; nenhum deles recebe paths de checkout.

A validação nativa limitada anterior no display primário confirmou cinco ciclos pelos controles, `Esc` em uma verificação separada, a geometria 16 × 96 ↔ 960 × 600 ancorada na borda direita e material com conteúdo de outro aplicativo visivelmente desfocado. A QA atual do refinamento confirmou uma janela, a fita única de 28 × 112, a folha de 960 × 600, a borda direita fixa e o centro preservado durante fechar e reabrir. Ela não mede o viewport diretamente nem valida foco entre aplicativos, clique nos cantos esquerdos e no aplicativo atrás, hover/reversão com cursor real, seleção e captura real de ponteiro, monitores mistos/desconexão ou preferências de acessibilidade. Os testes automatizados comprovam contratos e geometria, não esses cenários.

Durante o morph entre os raios 12 e 20, o polling usa provisoriamente o maior raio apenas nos cantos esquerdos. Isso evita reter um clique onde o vidro pode já estar transparente, mas pode também ignorar um canto ainda visível no começo da abertura. A equivalência entre a camada apresentada pelo AppKit e o hit-testing não foi comprovada; não há alegação de click-through para outro aplicativo.
