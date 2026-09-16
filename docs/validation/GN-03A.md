# Validação da autorização de raízes e preferências (GN-03A)

Ambiente local: macOS 26.6.2, Apple Silicon (arm64), Node 22.23.2, pnpm 10.32.1, Rust 1.98.1 e Git 2.50.1 (Apple Git-155).

O roteiro nativo histórico partiu de `c7d9894`. A integração atual acrescenta código de autorização, IPC e gaveta; o SHA revisado será registrado junto da QA nativa dessa combinação.

## Verificações locais

- `pnpm check`: Biome, typecheck e 8 testes Node passaram.
- `pnpm build`: assets estáticos gerados em `dist`.
- `pnpm rust:check`:
  - `cargo fmt --check`: formatação conforme o projeto.
  - `cargo clippy --locked --all-targets -- -D warnings`: zero advertências.
  - `cargo test --locked`: 66 testes aprovados no macOS, cobrindo settings, workspace, IPC com `tauri::test`, geometria e estado da gaveta.

O teste de diretório real com bytes não UTF-8 fica sob `cfg(target_os = "linux")`: APFS recusou criar essa fixture no macOS. Ele será exercitado pelo runner Linux; o teste local macOS cobre a fronteira de codificação usada antes da persistência.

## Cenários automatizados comprovados

1. **SettingsStore:** arquivo ausente vira padrão; roundtrip de gravação/leitura; JSON inválido recupera com aviso sem sobrescrever o arquivo; schema futuro bloqueia a escrita; identificadores duplicados, vazios ou não canônicos recuperam; `nextRootId` é elevado acima dos ids existentes e satura em `u64::MAX` sem descartar uma raiz histórica; escrita atômica substitui o conteúdo sem deixar temporário órfão; diretório ausente é criado.
2. **Workspace:** canonicalização do caminho na autorização (inclui `/tmp` → `/private/tmp` no macOS); deduplicação idempotente por caminho canônico sem incrementar a época; caminho não UTF-8 é recusado antes da persistência; ids e época exauridos falham sem escrever ou reutilizar estado; remoção revoga o handle e ele nunca revive (`r1` removido, próxima autorização cria `r2`); handle desconhecido; época divergente; raiz que sumiu do disco fica indisponível e é rejeitada na leitura; restauração entre instâncias; schema incompatível bloqueia mutações; raiz substituída por symlink é rejeitada.
3. **IPC com `tauri::test`:** a capability `notch` permite o webview declarado e recusa outro label e comando fora da allowlist antes do handler; caminho absoluto enviado como `rootId` é recusado como handle desconhecido, enquanto o id opaco `r1` funciona; handle removido é recusado; época antiga é recusada antes de qualquer leitura; guarda lexical de `rel_path` recusa `..`, caminho absoluto e string vazia; raiz indisponível é recusada; a view expõe época, health e raízes sem aceitar caminhos do webview.
4. **Prova de não mutação:** o teste `read_commands_do_not_mutate_the_repository` compara byte a byte `.git` e o worktree antes e depois de 5 ciclos de status e diff via IPC; os mapas são idênticos.
5. **Frontend:** o aceitador monotônico de `WorkspaceView` conserva a época mais recente entre carregamento, autorização, remoção e término de status.

## Roteiro nativo do painel integrado no macOS

Fixture temporária `/tmp/gitnotch-gn03a-fix/repo` (repositório Git com 1 arquivo modificado e 1 novo) e `/tmp/gitnotch-gn03a-fix/nao-repo` (pasta comum). Capturas locais em `artifacts/` (não versionadas).

1. Abrir a fita como prévia e acionar "Adicionar pasta": a gaveta deve fixar antes de o diálogo nativo abrir, portanto perda de foco e hover não podem recolhê-la.
2. Aplicativo sem preferências: a gaveta mostra "Nenhuma pasta autorizada ainda." sob o chrome existente.
3. "Adicionar pasta" abre o diálogo nativo; selecionar `repo` mostra o caminho canonicalizado `/private/tmp/gitnotch-gn03a-fix/repo` e a linha `main · 1 modificados · 1 novos`.
4. Adicionar `nao-repo` mostra "Não é um repositório Git.".
5. Encerrar e reabrir o aplicativo: as duas raízes são restauradas e o status é recarregado.
6. "Remover" na raiz `repo` remove a linha da interface; `settings.json` fica somente com `r2` e `nextRootId: 3`.
7. Remover a segunda raiz deixa `roots: []` e mantém `nextRootId: 3`, comprovando ids monotônicos.

As preferências ficam em `~/Library/Application Support/io.github.icaroaguiar.gitnotch/settings.json`; nenhuma escrita ocorreu dentro das raízes.

## Reproduzir

```bash
source .local/use-rust.sh
export DEVELOPER_DIR=/Library/Developer/CommandLineTools
pnpm install --frozen-lockfile
pnpm check
pnpm build
pnpm rust:check
pnpm desktop:build -- --locked
```

Para o roteiro nativo, crie uma fixture Git descartável, execute o binário gerado, adicione a pasta pelo diálogo, reinicie, remova a raiz e confira o `settings.json` do aplicativo.

## Limites da evidência

- Windows e Linux não foram validados nativamente.
- O roteiro nativo acima precisa ser repetido para a combinação atual de painel e gaveta; checks, build e bundle não substituem essa QA.
- O erro de acesso após remoção não aparece como mensagem na interface: a linha deixa de existir. A rejeição de handle removido e de requisição com época antiga está comprovada pelos testes IPC.
- O aceitador monotônico de respostas no frontend tem teste de contrato; a sequência assíncrona completa da interface ainda depende do roteiro nativo.
- Bloqueio de schema futuro e recuperação de arquivo inválido foram verificados por testes unitários, não manualmente na interface.
