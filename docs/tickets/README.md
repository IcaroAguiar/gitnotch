# Tickets para entregar uma PR por vez

GN-00 está em avaliação local, com remoto e PR pendentes. Os demais tickets estão planejados e aguardam aceite do anterior. Uma linha representa uma PR planejada, não uma PR publicada.

A sequência é deliberadamente linear para permitir avaliação entre entregas. As dependências abaixo são também portões de revisão. O backlog original continua sendo a referência de cobertura.

| Ticket | Entrega | Aguarda merge | Origem |
|---|---|---|---|
| [GN-00](GN-00.md) | Preparar Git e regras | Nenhum | Preparação |
| [GN-01A](GN-01A.md) | Criar aplicação mínima e checks | GN-00 | GN-01 |
| [GN-01B](GN-01B.md) | Validar aba e gaveta nativas | GN-01A | GN-01 |
| [GN-01C](GN-01C.md) | Validar material e patch estático | GN-01B | GN-01 |
| [GN-02A](GN-02A.md) | Executar Git com ambiente e recursos limitados | GN-01C | GN-02 |
| [GN-02B](GN-02B.md) | Barrar filtros e programas externos | GN-02A | GN-02 |
| [GN-02C](GN-02C.md) | Interpretar estados Git sem perder caminhos | GN-02B | GN-02 |
| [GN-02D](GN-02D.md) | Ler comparações e estados especiais | GN-02C | GN-02 |
| [GN-03A](GN-03A.md) | Autorizar raízes e persistir preferências | GN-02D | GN-03 |
| [GN-03B](GN-03B.md) | Descobrir checkouts progressivamente | GN-03A | GN-03 |
| [GN-04A](GN-04A.md) | Navegar pelos grupos e arquivos | GN-03B | GN-04 |
| [GN-04B](GN-04B.md) | Mostrar o diff real sob demanda | GN-04A | GN-04 |
| [GN-05A](GN-05A.md) | Planejar watches e limitar fila | GN-04B | GN-05 |
| [GN-05B](GN-05B.md) | Recuperar eventos perdidos e suspensão | GN-05A | GN-05 |
| [GN-06A](GN-06A.md) | Distinguir baseline de atividade observada | GN-05B | GN-06 |
| [GN-06B](GN-06B.md) | Manter leitura estável e cache limitado | GN-06A | GN-06 |
| [GN-07A](GN-07A.md) | Aplicar preferências e acessibilidade | GN-06B | GN-07 |
| [GN-07B](GN-07B.md) | Concluir interação e geometria de desktop | GN-07A | GN-07 |
| [GN-08A](GN-08A.md) | Medir e fechar regressões da v0.1 | GN-07B | GN-08 |
| [GN-08B](GN-08B.md) | Gerar pacotes e documentar suporte | GN-08A | GN-08 |

## Portões de avaliação

- GN-01C fecha a prova inicial de janela/material no ambiente testado.
- GN-02D fecha a leitura Git antes de conectá-la à UI.
- GN-04B entrega o fluxo completo com atualização manual.
- GN-06B entrega atualização automática e leitura estável.
- GN-08B encerra somente os ambientes efetivamente validados.

Se um ticket ainda exceder uma alteração verificável, proponha uma divisão antes de ampliar o diff. Preserve o ID de origem e explicite a dependência. Não elimine casos da especificação para caber em uma PR.
