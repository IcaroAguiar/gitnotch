# Git Notch

Utilitário desktop local para acompanhar alterações de vários checkouts Git em uma gaveta lateral, com visualização de diffs somente de leitura.

## Estado do projeto

Há uma aba e uma gaveta nativas em macOS, com estado compartilhado no Rust, e um leitor Git somente de leitura coberto por testes. A navegação de repositórios ainda não está conectada à interface e material/blur ficam para a próxima etapa. Não há release publicada.

Tauri 2, React/TypeScript e Rust. O frontend é empacotado localmente e abre sem servidor de desenvolvimento. O Git instalado será a fonte de estado dos checkouts quando a leitura for conectada à interface.

## Escopo

- Descobrir checkouts dentro de pastas escolhidas pelo usuário.
- Separar staged, unstaged, arquivos novos e conflitos.
- Mostrar diffs sob demanda e atividade efetivamente observada.
- Preservar os repositórios observados, sem stage, commit ou alterações de configuração.

Editor, terminal, conta, sincronização, telemetria remota e atualizador automático estão fora da v0.1. Esses são limites do projeto planejado, não garantias de uma implementação já testada.

## Documentação

- [Especificação v0.1](git-notch-spec-v0.1.md).
- [Política de contribuições](CONTRIBUTING.md).
- [Regras para agentes](AGENTS.md).
- [Registros de validação](docs/validation.md).

## Compilar

Use Node na versão de `.node-version`, pnpm na versão de `packageManager` em `package.json` e Rust via rustup. `rust-toolchain.toml` seleciona a versão e os componentes Rust. Instale os [pré-requisitos nativos do Tauri](https://v2.tauri.app/start/prerequisites/) para seu sistema.

```sh
pnpm install --frozen-lockfile
pnpm check
pnpm build
pnpm rust:check
pnpm desktop:build -- --locked
```

`pnpm check` executa lint, typecheck e testes do contrato de configuração desktop. `pnpm rust:check` executa rustfmt, Clippy e o test runner Rust, que cobre a geometria e o estado da gaveta e o leitor Git. Execute `pnpm build` antes dos checks Rust, pois o contexto Tauri incorpora os assets de dist.

Para gerar um pacote nativo local:

```sh
pnpm desktop:bundle -- --locked
```

O Tauri gera os artefatos em `src-tauri/target/release/`. No macOS, o aplicativo está em `src-tauri/target/release/bundle/macos/Git Notch.app`. Abra esse `.app` pelo Finder e encerre com Cmd+Q. O pacote local não é uma release assinada/notarizada para distribuição.

Para iterar, edite o código e repita o build. Este fluxo não inicia servidor persistente. `pnpm format` aplica a formatação e as correções automáticas do Biome.

Os builds da CI não publicam releases. Consulte [a validação](docs/validation.md) para distinguir compilação de teste nativo.

## Projeto pessoal

O código é disponibilizado para uso e adaptação sob a licença MIT. Não aceitamos contribuições externas ou PRs não solicitadas e não oferecemos compromisso de suporte. O desenvolvimento é conduzido pelo mantenedor.

## Licença

Distribuído sob a [licença MIT](LICENSE). As licenças das dependências permanecem aplicáveis aos respectivos componentes.
