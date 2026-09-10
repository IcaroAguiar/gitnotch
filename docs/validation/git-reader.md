# Validação do executor Git de leitura e política de segurança (GN-02)

Ambiente local: macOS 26.6.2, Apple Silicon, Node 22.23.2, pnpm 10.32.1, Rust 1.98.1 e Git 2.39.5 (Apple Git-154).

## Verificações locais

- `pnpm check`: biome lint, typecheck e testes contratuais de configuração passaram sem erros.
- `pnpm rust:check`:
  - `cargo fmt --check`: formatação conforme padrões do projeto.
  - `cargo clippy --locked --all-targets -- -D warnings`: zero advertências de linter.
  - `cargo test --locked`: 10 testes de integração executados em fixtures temporárias descartáveis, todos aprovados (10 passed; 0 failed).
- `pnpm desktop:build`: build de produção executado com sucesso, gerando o executável final em `src-tauri/target/release/gitnotch`.

## Cenários de aceitação comprovados

1. **Capacidades e localização do Git:** detecção automática e validação de versão mínima (>= 2.22) e suporte a porcelain v2 sem shell.
2. **Limite de recursos e cancelamento:** comandos com volume de saída acima do limite têm o subprocesso finalizado e colhido sem processos zumbis ou travamentos de pipe.
3. **Repositório Unborn:** novo repositório sem commits tratados corretamente; staged reportado como adição e diff gerado contra a árvore vazia sem erro de HEAD inexistente.
4. **Detached HEAD:** identificação explícita de `(detached)` e commit correspondente.
5. **Mesmo arquivo nos dois grupos:** arquivo com modificações staged e unstaged simultâneas (`MM`) aparece em ambos os grupos, produzindo diffs específicos para cada estado.
6. **Caminhos literais especiais:** arquivos com espaços, dois pontos, colchetes e acentuação são tratados literalmente sem interpretação mágica de pathspecs.
7. **Renomeações:** preservação do caminho atual e do caminho original, gerando diff de rename compatível.
8. **Conflitos de merge:** arquivos não resolvidos agrupados explicitamente como conflitados.
9. **Bloqueio de filtros externos:** filtro `clean` sintético associado via `.gitattributes` a arquivos rastreados é detectado pelo preflight; a consulta é classificada como `LimitedByExternalFilter` e o comando externo nunca é executado (provado por ausência de criação de arquivo marcador).
10. **Prova de não-mutação:** comparação byte a byte de `.git/HEAD`, `.git/index`, `.git/config`, referências e arquivos do worktree antes e depois de múltiplas consultas confirma ausência total de escritas nos repositórios observados.

## Reproduzir

```bash
source .local/use-rust.sh
pnpm check
pnpm rust:check
pnpm desktop:build
```

## Limites da evidência

Os testes comprovam a fidelidade e segurança do backend Rust via subprocessos Git reais e comandos IPC. A integração visual da gaveta e a navegação multi-repositório dependem dos tickets GN-03 e GN-04 e ainda não foram testadas na interface gráfica nesta etapa.
