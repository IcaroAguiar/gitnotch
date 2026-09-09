# GN-06B: manter leitura estável e cache limitado

Estado: planejado. Dependência: GN-06A aceito e incorporado à main. Origem: GN-06 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-06 em `../../git-notch-backlog-v0.1.md` e as seções 7, 11, 12, 13, 14 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src/drawer/, src/diff/, cache Rust. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Preservar ordem, expansões e âncora. Cache LRU por identidade e revisão com orçamento normativo. Invalidar quando inputs mudarem e mostrar atualização pendente se aplicar patch deslocaria leitura.

## Critérios obrigatórios

Arquivo fica limpo sem mudar seleção. Burst não reordena lista durante leitura. Resposta obsoleta não entra em seleção nova. Cache respeita primeiro limite atingido e remove dados revogados.

## Teste do avaliador

Ler diff longo, editar externamente, fechar/reabrir e remover raiz. Registrar posição e uso do cache.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-06B.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem manter histórico de patches em disco.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
