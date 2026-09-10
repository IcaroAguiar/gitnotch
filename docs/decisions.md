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
