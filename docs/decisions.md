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

## Autorização de raízes e preferências (GN-03A)

A seleção de pastas usa o diálogo nativo do `tauri-plugin-dialog` a partir do Rust. O Rust canonicaliza a pasta escolhida, exige que seja um diretório e rejeita nomes que não podem ser codificados em UTF-8 antes de persistir preferências. A raiz recebe um identificador opaco `r{n}`. O IPC não aceita caminhos absolutos: `get_repo_status` e `get_file_diff` recebem apenas `root_id` e recusam handles desconhecidos, removidos ou com época divergente. A guarda lexical de `rel_path` continua barrando `..` e caminhos absolutos antes de qualquer leitura.

As preferências ficam em um JSON versionado (`schemaVersion: 1`) no diretório de configuração do aplicativo, com gravação atômica por arquivo temporário no mesmo diretório, `sync_all` e `rename`. Arquivo ausente vira padrão; JSON inválido é recuperado com aviso sem sobrescrita imediata; schema mais novo que o suportado bloqueia a escrita e reporta incompatibilidade; ids são validados e normalizados na carga. O maior id histórico satura o próximo id em `u64::MAX`, preservando a raiz existente para leitura ou remoção. Autorizar outra raiz ou alterar uma época esgotada retorna erro antes de gravar ou alterar memória.

A época de autorização (`epoch`) vive apenas em memória, começa em 1 e incrementa ao autorizar (fora de deduplicação) ou remover. A remoção grava antes de alterar a memória, revoga o handle e invalida requisições pendentes: cada envelope de resposta carrega a época da resolução. Um único aceitador monotônico no painel protege carregamento, autorização, remoção e status para que uma resposta de época anterior não substitua a view atual. Como os ids são monotônicos e persistidos, um handle removido nunca é reutilizado.

A capability `notch` é ligada ao único webview local e enumera cada comando de raiz, Git e gaveta, além de escutar o evento de estado. `AppManifest` gera as permissões dos comandos customizados; um label diferente ou comando fora da lista é recusado antes do handler. A CSP permite apenas `connect-src 'self' ipc: http://ipc.localhost`, necessário ao transporte IPC do Tauri 2.11.

Referências consultadas:

- [Plugin dialog](https://v2.tauri.app/plugin/dialog/).
- [Chamar Rust do frontend](https://v2.tauri.app/develop/calling-rust/).
- [Permissões Tauri](https://v2.tauri.app/security/permissions/).
## Aba e gaveta nativas (GN-01B)

A implementação usa **uma forma Tauri nativa** com o label `notch`, em vez de duas janelas/WebViews. Ela nasce como fita de 28 × 112 lógicos e, quando a intenção muda para `preview` ou `pinned`, cresce para até 960 × 600 lógicos. A fita fechada fica encostada à direita da área útil; a gaveta aberta flutua 24 px lógicos antes dela e recebe quatro cantos de 20 px. Em uma área menor, ela reduz largura e altura sem ultrapassar a área útil e mantém margens laterais equilibradas quando houver espaço. A escolha reduz o custo e a ambiguidade de foco de uma segunda janela, mas ainda requer aceite nativo para comportamento real de foco e hit-testing.

A referência visual escolhida é a fita de borda do [Yoink](https://eternalstorms.at/yoink/mac/) e o acesso persistente do [SideNotes](https://www.apptorium.com/sidenotes). A decisão traduz somente a relação espacial: uma alça estreita encostada à direita que se torna uma gaveta flutuante para a esquerda, com uma alça compacta integrada. Não reaproveita código, assets ou comportamento desses produtos.

A área útil vem primeiro de `current_monitor` da forma e usa o monitor primário apenas como fallback. Os retângulos de Tauri usam pixels físicos. No AppKit, o adaptador deriva o frame-alvo da moldura atual de `NSWindow` e da posição externa atual da própria forma, em vez de converter pela altura de `NSScreen.mainScreen`. Isso preserva origem negativa e escala do monitor atual no cálculo; o teste unitário cobre uma origem negativa com escala 2. A geometria real em monitores mistos continua pendente de validação nativa.

O Rust é a fonte de verdade do estado da gaveta:

- `DrawerIntent`: `closed`, `preview` e `pinned`;
- `DrawerPhase`: `resting`, `opening` e `closing`;
- geração monotônica para cada intenção que muda a apresentação.

Uma intenção nova invalida a conclusão anterior. Fixar durante a abertura cria uma nova transição `pinned/opening`, então a conclusão da prévia não pode assentar o painel. O frontend aceita apenas snapshots de geração mais recente e, na mesma geração, recusa uma fase animada recebida depois de `resting`.

A checagem da geração acontece no main thread imediatamente antes de foco, evento e frame nativos. O mutex permanece retido somente enquanto o efeito é enfileirado; a conclusão de `NSAnimationContext` agenda a atualização de fase depois dessa região crítica. Isso evita o temporizador estimado que existia antes e rejeita callbacks obsoletos, sem afirmar que a continuidade visual de uma interrupção já foi aceita em hardware.

O hover abre após 120 ms de permanência e a prévia tolera 250 ms de saída. Durante a prévia, um corredor de hover limitado à altura original da fita cobre somente a lacuna até a borda direita; ele não altera a moldura ou o hit-testing da janela. A implementação atual abre em 380 ms com `cubic-bezier(.16,1,.3,1)` e recolhe em 240 ms com `cubic-bezier(.32,.72,0,1)`; o candidato anterior documentado em GN-01B usava 280/200 ms. Reduzir movimento entrega duração nativa zero e remove os atrasos CSS. `InteractionGuards` suspende o recolhimento automático durante seleção e durante captura real de ponteiro. O frontend espelha seleção, `gotpointercapture`/`lostpointercapture` e limpa a guarda em `pointerup`, cancelamento ou perda de foco; ele não captura artificialmente ponteiros de botões.

No macOS, o material usa `NSGlassEffectView` público no estilo `Regular` dentro de uma raiz `NSView` com o frame visível da janela e `clipsToBounds` ativo. Em `closed/resting`, o vidro e seu host de conteúdo excedem 20 pontos à direita para manter a fita com lado direito reto; na abertura, o excedente anima a zero, expondo os quatro cantos do vidro. O conteúdo Tauri preserva sempre a largura visível da raiz, com reconciliação explícita de frame e `autoresizing` ao final da animação. O adaptador nunca guarda ponteiro Objective-C cru: localiza vidro por raiz de recorte e restaura o conteúdo original do host ao cair no fallback sólido. A leitura do raio ocorre no main thread sob o mutex imediatamente antes de instalar o efeito. A redução de transparência força sólido. No candidato atual, `NSAppearanceNameAqua` é aplicado somente ao `NSGlassEffectView` enquanto a forma está aberta ou fechando; ao voltar a `closed/resting`, o adaptador restaura `appearance = nil` para herdar o sistema. Não define aparência em `NSWindow` nem no sistema; as subviews do vidro podem herdar Aqua. A janela não ativa sombra nativa adicional, porque o candidato com sombra produziu artefatos pretos nos cantos inferiores. O preenchimento CSS mantém uma única fita DOM sobre o conteúdo e agora abriga o painel de raízes autorizadas, sem inventar repositórios, arquivos ou diffs.

A capability `notch` permite somente o evento de estado e os comandos explicitamente declarados para raízes, Git e gaveta. Os comandos de leitura recebem ids de raiz opacos; nenhum comando aceita paths de checkout arbitrários.

A QA nativa limitada atual no display primário confirmou uma janela, a fita de 28 × 112 encostada à borda, a gaveta de 960 × 600 a 24 px da direita, os quatro cantos do vidro abertos e a alça translúcida. `Esc`, clique na alça, Recolher e reabertura em ciclos foram observados. Ela não mede o viewport ou a largura do conteúdo diretamente nem valida hover/reversão com cursor físico, foco entre aplicativos, clique nos cantos arredondados e no aplicativo atrás, seleção e captura real de ponteiro, monitores mistos/desconexão ou preferências de acessibilidade. Os testes automatizados comprovam contratos e geometria, não esses cenários.

Durante o morph entre os raios 12 e 20, o polling usa provisoriamente o maior raio nos quatro cantos quando a gaveta está aberta ou em transição e somente nos cantos esquerdos quando está fechada. Isso evita reter um clique onde o vidro pode já estar transparente, mas pode também ignorar um canto ainda visível no começo da abertura. A equivalência entre a camada apresentada pelo AppKit e o hit-testing não foi comprovada; não há alegação de click-through para outro aplicativo.

## Drawer flutuante (GN-01B, 16/09/2026)

O candidato baseado em `f321d35976d432f2fc7408004d658d60fb4668ee` escolhe uma única janela que se desprende 24 px da borda somente aberta, em vez de criar uma ponte visual ou uma segunda janela. O excedente do material muda de 20 para 0 pontos junto da transição, enquanto o conteúdo continua limitado ao recorte visível. Isso recupera os cantos direitos do vidro sem alargar o WebView.

O experimento de `NSWindow.hasShadow` foi removido: no candidato anterior ele deixou uma borda e triângulos pretos nos cantos inferiores. O candidato de QA mantém `shadow: false` e não adiciona sombra CSS fora da janela. As evidências de candidatos anteriores permanecem em `docs/validation/GN-01B.md`; esta decisão não representa aceite visual do usuário.
