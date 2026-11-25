# Validação das ferramentas de build e lint

Base: `301c4b8`. Código verificado: o commit desta alteração.
Ambiente: macOS/Apple Silicon, Node 22.23.5 e pnpm 10.32.1. O projeto não tem `.nvmrc`; `.node-version` indica 22.23.2. A execução usou o patch 22.23.5 já instalado, dentro de `engines.node`.

## Escopo

Biome 2.6.6 → 2.6.8 (publicado em 15/11/2026). O schema do Biome e a relação de dependências acompanham a versão instalada. Vite, plugin React e `@types/node` permanecem como estavam.

A atualização do Tauri continua adiada: API e CLI 2.12.5 exigem major/minor iguais aos do crate Rust `tauri` (2.11.5, fixado em `Cargo.toml`), e o CLI recusa o build com "Found version mismatched Tauri packages". API 2.11.1, CLI 2.11.4 e crates/lockfile Rust foram preservados. O ambiente não dispõe de `cargo`, `rustc` ou `rustup`.

## Verificações automatizadas

Com Node 22 no PATH, a partir da raiz:

| Comando | Resultado |
|---|---|
| `heavy pnpm install --frozen-lockfile` | Passou; lockfile consistente |
| `heavy pnpm lint` | Passou; os mesmos dois avisos `noDescendingSpecificity` em `src/styles.css` que a base já tinha |
| `heavy pnpm typecheck` | Passou |
| `heavy pnpm test` | Passou; oito testes, sem skips |
| `heavy pnpm build` | Passou; 23 módulos, assets estáticos em `dist/` |
| `heavy pnpm check` | Passou (lint, typecheck e testes) |

## Limites

Não houve execução da janela nativa, build/testes Rust ou validação em Windows/Linux.
