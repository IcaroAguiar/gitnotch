# Git Notch

Utilitário desktop local para acompanhar alterações de vários checkouts Git em uma gaveta lateral, com visualização de diffs somente de leitura.

## Estado do projeto

O projeto está em preparação. Este repositório contém a especificação, o plano de implementação e as regras de contribuição. Ainda não há aplicativo executável, instalador ou plataforma validada.

A stack planejada é Tauri 2, React/TypeScript e Rust. O frontend será empacotado localmente. O Git instalado será a fonte de estado dos checkouts.

## Escopo

- Descobrir checkouts dentro de pastas escolhidas pelo usuário.
- Separar staged, unstaged, arquivos novos e conflitos.
- Mostrar diffs sob demanda e atividade efetivamente observada.
- Preservar os repositórios observados, sem stage, commit ou alterações de configuração.

Editor, terminal, conta, sincronização, telemetria remota e atualizador automático estão fora da v0.1. Esses são limites do projeto planejado, não garantias de uma implementação já testada.

## Documentação

- [Especificação v0.1](git-notch-spec-v0.1.md).
- [Backlog por marco](git-notch-backlog-v0.1.md).
- [Tickets de implementação](docs/tickets/README.md).
- [Como contribuir](CONTRIBUTING.md).
- [Regras para agentes](AGENTS.md).
- [Registros de validação](docs/validation.md).

Os comandos de desenvolvimento serão adicionados com a aplicação mínima em GN-01A. Nenhuma instalação global é necessária para ler ou revisar esta documentação.

## Licença

Distribuído sob a [licença MIT](LICENSE). As licenças das dependências permanecem aplicáveis aos respectivos componentes.
