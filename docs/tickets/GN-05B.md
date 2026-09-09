# GN-05B: recuperar eventos perdidos e suspensão

Estado: planejado. Dependência: GN-05A aceito e incorporado à main. Origem: GN-05 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-05 em `../../git-notch-backlog-v0.1.md` e as seções 10.2, 14, 16.2 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

watch.rs, reconciliação, estado degradado no frontend. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Adicionar reconciliação escalonada, redescoberta periódica, retorno de suspensão, backoff, atualização manual e polling declarado quando necessário.

## Critérios obrigatórios

Descartar evento artificialmente e demonstrar convergência. Simular falha/retry, remover/criar repo e reabrir após suspensão. Falha de um repo não trava os demais.

## Teste do avaliador

Modificar fixture sem evento entregue e aguardar reconciliação no intervalo documentado. Testar refresh manual durante backoff.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-05B.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem prometer confiabilidade de filesystem remoto não testado.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
