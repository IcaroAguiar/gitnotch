# GN-03B: descobrir checkouts progressivamente

Estado: planejado. Dependência: GN-03A aceito e incorporado à main. Origem: GN-03 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-03 em `../../git-notch-backlog-v0.1.md` e as seções 8, 12, 14, 16.1 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src-tauri/src/discovery.rs, registry, UI mínima de raízes. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Descobrir repos aninhados e .git arquivo/diretório, worktrees e submódulos. Deduplicar raízes sobrepostas preservando checkout. Aplicar exclusões, limites e cancelamento, com progresso e erros isolados.

## Critérios obrigatórios

Fixture em três profundidades, nomes iguais, worktrees com metadados externos e exclusão autorizada explicitamente. Limite mostra descoberta parcial. Remover raiz interrompe jobs e nega resultados tardios.

## Teste do avaliador

Selecionar fixture e comparar a lista com inventário esperado. Cancelar descoberta e remover raiz durante execução.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-03B.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem varredura fora de raízes salvo metadados/configs estritamente necessários, sem seguir symlinks indiscriminadamente.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
