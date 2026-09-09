# GN-06A: distinguir baseline de atividade observada

Estado: planejado. Dependência: GN-05B aceito e incorporado à main. Origem: GN-06 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-06 em `../../git-notch-backlog-v0.1.md` e as seções 11, 12, 16.2 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src-tauri/src/activity.rs, eventos e indicador da aba. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Registrar baseline sem horário fictício. Distinguir evento, fingerprint alterado e diff efetivamente novo. Contar repos com novidades, com set_seen_revision condicionado à revisão lida.

## Critérios obrigatórios

Editar arquivo já M mantém status mas sinaliza atividade correta. Touch sem conteúdo não afirma novo diff. Nova revisão durante leitura continua não vista.

## Teste do avaliador

Abrir com alterações preexistentes, editar de novo e comparar o indicador antes/depois de ler.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-06A.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem atribuição a agentes, tarefas ou aprovação das alterações.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
