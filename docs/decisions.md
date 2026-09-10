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

## Executor Git de leitura e política de segurança (GN-02)

O módulo `GitReader` implementa a fonte de verdade baseada no Git instalado no sistema operacional, sem invocar shell, login shell ou instalar executáveis. A localização do binário ocorre uma vez no PATH e valida que a versão seja igual ou superior a 2.22, garantindo suporte completo ao formato porcelain v2 e às flags de segurança.

Todas as chamadas Git utilizam subprocessos com listas explícitas de argumentos e guardas obrigatórias:
`--no-optional-locks`, `--no-lazy-fetch`, `--literal-pathspecs`, `-c core.fsmonitor=false`, `-c protocol.allow=never`. O ambiente herdado é limpo de variáveis `GIT_*` e `PAGER`, e são impostas variáveis determinísticas de segurança (`GIT_TERMINAL_PROMPT=0`, `GIT_OPTIONAL_LOCKS=0`, `GIT_NO_LAZY_FETCH=1`).

O ciclo de vida dos subprocessos protege contra esgotamento de recursos e deadlocks de pipe: stdout e stderr são drenados concorrentemente por threads dedicadas com teto estrito de bytes (20 MiB por padrão) e timeout (15 s por padrão). O estouro de limite ou tempo encerra o processo via sinal e colhe o processo (`wait`), sem deixar processos zumbis ou órfãos.

Filtros externos (`clean` ou `process`) podem executar código arbitrário durante leituras do worktree. Conforme a seção 9.3 da especificação, implementou-se preflight que inspeciona a configuração efetiva (`filter.*.clean` e `filter.*.process`) e cruza com os atributos dos arquivos rastreados via `ls-files -z` e `check-attr -z filter --stdin`. Quando um filtro externo se aplica a arquivos rastreados, o repositório é classificado como `LimitedByExternalFilter`, recusando consultas de worktree que acionem o filtro e preservando o isolamento do produto.

O parsing utiliza `status --porcelain=v2 --branch -z --no-ahead-behind --untracked-files=all --find-renames=50%`, tratando bytes e delimitadores NUL sem quebra por espaços ou tabs. Um mesmo arquivo com modificações staged e unstaged (`MM`) é categorizado em ambos os grupos. Repositórios `unborn` e `detached HEAD` são tratados explicitamente. Diffs de arquivos não rastreados são gerados por leitura direta delimitada, sem `git add` e sem criar arquivos temporários dentro do repositório.

## Aba e gaveta nativas (GN-01B)

A estratégia de duas janelas pequenas da seção 4.1 foi mantida. A aba usa 28 × 64 lógicos, sem foco, sempre no topo e visível em todos os espaços; a gaveta usa 600 × 800 lógicos, começa oculta e é reaproveitada enquanto o aplicativo estiver aberto. As duas carregam o mesmo bundle e o frontend roteia pelo label. A decisão considerou foco e hit-testing aprovados, 20 ciclos estáveis e memória sem crescimento com a gaveta. A alternativa de uma janela redimensionável permanece documentada, atrás do mesmo adaptador, caso a medição comparável de GN-08A reprove o custo do segundo WebView. Os dois WebViews já existem desde a inicialização; o custo incremental do segundo é um processo WebContent a mais, na faixa de 16 a 34 MiB conforme a métrica.

Transparência no macOS exige a API privada `macOSPrivateApi` e a feature `macos-private-api`. A distribuição prevista na seção 17 é local e notarizada, não App Store, então a restrição de revisão da Apple não se aplica. Windows e Linux seguem sem transparência dependente dessa feature.

A geometria fica em funções puras em `desktop.rs`. A aba encosta na borda direita da área útil e centraliza na vertical; a gaveta encosta na esquerda da aba com margem de 12, centraliza na aba e é limitada à área útil, com altura máxima de 800 e recorte de 24 da altura útil. Tamanhos lógicos são escalados pelo `scale_factor` do monitor e a posição final usa pixels físicos.

O estado `closed` e `open` vive no Rust com geração monotônica. Perder foco recolhe a gaveta e um novo clique na aba em até 250 ms não a reabre, porque o mesmo gesto gera blur e clique. A máquina completa com `opening` e `closing` fica para GN-07.

A CSP passou de `connect-src 'none'` para `connect-src 'self' ipc: http://ipc.localhost`, necessária para o transporte IPC do Tauri 2.11. A política continua restrita a recursos locais.

A aba fica grudada na borda direita da tela, com cantos arredondados apenas no lado exposto e raio de 12; a gaveta usa raio de 20. Onde `corner-shape: superellipse()` existir, a curvatura vira squircle, mas o WKWebView atual ainda não suporta a propriedade e o build testado usa `border-radius` circular.

O movimento leve de abrir e recolher foi antecipado de GN-07 a pedido do mantenedor e usa tokens no CSS: hover de 110 ms, abrir de 220 ms com `cubic-bezier(0.2, 0.8, 0.2, 1)`, recolher de 170 ms com `cubic-bezier(0.4, 0, 1, 1)` e fade de 80 ms sob movimento reduzido. A animação é de conteúdo dentro da janela, não de geometria nativa, conforme o limite da seção 6. O Rust emite `gitnotch://drawer-state` com estado e geração, espera 180 ms e só oculta a janela se a geração ainda for a mais recente; uma nova abertura cancela o fechamento pendente. A gaveta recebe uma capability que concede apenas `core:event:allow-listen` e `core:event:allow-unlisten`.

Referências de movimento consultadas: HIG de Motion, WWDC18 803 (Designing Fluid Interfaces) e WWDC23 10158 (Animate with springs).
