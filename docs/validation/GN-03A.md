# Validação da autorização de raízes e preferências (GN-03A)

Ambiente local: macOS 26.6.2, Apple Silicon (arm64), Node 22.23.2, pnpm 10.32.1, Rust 1.98.1 e Git 2.50.1 (Apple Git-155).

Código verificado: `c7d9894`. Alterações posteriores deste registro são apenas documentação.

## Verificações locais

- `pnpm check`: Biome, typecheck e 2 testes do contrato de configuração desktop passaram.
- `pnpm build`: assets estáticos gerados em `dist`.
- `pnpm rust:check`:
  - `cargo fmt --check`: formatação conforme o projeto.
  - `cargo clippy --locked --all-targets -- -D warnings`: zero advertências.
  - `cargo test --locked`: 35 testes aprovados (10 herdados do GN-02 e 25 novos: settings, workspace e IPC com `tauri::test`).
- `pnpm desktop:build -- --locked`: binário gerado em `src-tauri/target/release/gitnotch`. A primeira tentativa falhou por falta de espaço em disco (`No space left on device`, os error 28); a repetição após liberar o cache local de build em `src-tauri/target/debug` concluiu sem erro de código.

## Cenários automatizados comprovados

1. **SettingsStore:** arquivo ausente vira padrão; roundtrip de gravação/leitura; JSON inválido recupera com aviso sem sobrescrever o arquivo; schema futuro bloqueia a escrita; identificadores duplicados, vazios ou não canônicos recuperam; `nextRootId` é elevado acima dos ids existentes; escrita atômica substitui o conteúdo sem deixar temporário órfão; diretório ausente é criado.
2. **Workspace:** canonicalização do caminho na autorização (inclui `/tmp` → `/private/tmp` no macOS); deduplicação idempotente por caminho canônico sem incrementar a época; remoção revoga o handle e ele nunca revive (`r1` removido, próxima autorização cria `r2`); handle desconhecido; época divergente; raiz que sumiu do disco fica indisponível e é rejeitada na leitura; restauração entre instâncias; schema incompatível bloqueia mutações; raiz substituída por symlink é rejeitada.
3. **IPC com `tauri::test`:** caminho absoluto enviado como `rootId` é recusado como handle desconhecido, enquanto o id opaco `r1` funciona; handle removido é recusado; época antiga é recusada antes de qualquer leitura; guarda lexical de `rel_path` recusa `..`, caminho absoluto e string vazia; raiz indisponível é recusada; a view expõe época, health e raízes sem aceitar caminhos do webview.
4. **Prova de não mutação:** o teste `read_commands_do_not_mutate_the_repository` compara byte a byte `.git` e o worktree antes e depois de 5 ciclos de status e diff via IPC; os mapas são idênticos.

## Roteiro nativo no macOS

Fixture temporária `/tmp/gitnotch-gn03a-fix/repo` (repositório Git com 1 arquivo modificado e 1 novo) e `/tmp/gitnotch-gn03a-fix/nao-repo` (pasta comum). Capturas locais em `artifacts/` (não versionadas).

1. Aplicativo sem preferências: painel vazio com "Nenhuma pasta autorizada ainda.".
2. "Adicionar pasta" abre o diálogo nativo; selecionar `repo` mostra o caminho canonicalizado `/private/tmp/gitnotch-gn03a-fix/repo` e a linha `main · 1 modificados · 1 novos`.
3. Adicionar `nao-repo` mostra "Não é um repositório Git.".
4. Encerrar e reabrir o aplicativo: as duas raízes são restauradas e o status é recarregado.
5. "Remover" na raiz `repo` remove a linha da interface; `settings.json` fica somente com `r2` e `nextRootId: 3`.
6. Remover a segunda raiz deixa `roots: []` e mantém `nextRootId: 3`, comprovando ids monotônicos.

As preferências ficam em `~/Library/Application Support/io.github.icaroaguiar.gitnotch/settings.json`; nenhuma escrita ocorreu dentro das raízes.

## Reproduzir

```bash
source .local/use-rust.sh
pnpm install --frozen-lockfile
pnpm check
pnpm build
pnpm rust:check
pnpm desktop:build -- --locked
```

Para o roteiro nativo, crie uma fixture Git descartável, execute o binário gerado, adicione a pasta pelo diálogo, reinicie, remova a raiz e confira o `settings.json` do aplicativo.

## Limites da evidência

- Windows e Linux não foram validados nativamente.
- O diálogo nativo foi exercitado por automação de acessibilidade, sem rodada manual humana. O avaliador deve repetir o roteiro.
- O erro de acesso após remoção não aparece como mensagem na interface: a linha deixa de existir. A rejeição de handle removido e de requisição com época antiga está comprovada pelos testes IPC.
- O descarte de respostas com época divergente no frontend foi implementado, mas não tem teste automatizado de interface; o contrato Rust correspondente é testado.
- Bloqueio de schema futuro e recuperação de arquivo inválido foram verificados por testes unitários, não manualmente na interface.
