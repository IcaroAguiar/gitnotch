# Validação das ferramentas de build e lint

Código verificado: `5d80ce9`. Base: `c0b199a`.
Ambiente: macOS/Apple Silicon, Node 22.23.5 e pnpm 10.32.1. O projeto não tem `.nvmrc`; `.node-version` indica 22.23.2. A execução usou o patch 22.23.5 já instalado, dentro de `engines.node`.

## Escopo

Vite 8.3.5 → 8.3.8, plugin React 6.2.1 → 6.3.0 e Biome 2.6.4 → 2.6.6. O schema do Biome e a relação de dependências acompanham as versões instaladas. `@types/node` permanece em 22.20.1.

A atualização do Tauri foi adiada: a [CLI 2.12.4 exige major/minor iguais entre o crate e a API JavaScript](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-cli/src/info/plugins.rs). Atualizar somente a API para 2.12.4 deixaria o crate 2.11.5 incompatível com essa checagem. O ambiente não dispõe de `cargo`, `rustc` ou `rustup` para validar o alinhamento Rust. API 2.11.1, CLI 2.11.4 e crates/lockfile Rust foram preservados.

## Verificações automatizadas

Com Node 22 no PATH, a partir da raiz:

| Comando | Resultado |
|---|---|
| `heavy pnpm install --frozen-lockfile` | Passou; lockfile consistente |
| `heavy pnpm lint` | Passou; dois avisos `noDescendingSpecificity` em `src/styles.css`, linhas 93 e 529 |
| `heavy pnpm typecheck` | Passou |
| `heavy pnpm test` | Passou; oito testes, sem skips |
| `heavy pnpm build` | Passou; 23 módulos, assets estáticos em `dist/` |

Os avisos do lint não foram corrigidos nesta alteração de dependências. Não houve falha de verificação que exigisse comparação de execução com a base.

## Limites

O build frontend comprova a geração dos assets com Vite 8.3.8. Não houve execução da janela nativa, build/testes Rust ou validação em Windows/Linux. Esses resultados não comprovam interação nativa nem uma release distribuída.
