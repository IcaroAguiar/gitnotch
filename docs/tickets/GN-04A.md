# GN-04A: navegar pelos grupos e arquivos

Estado: planejado. Dependência: GN-03B aceito e incorporado à main. Origem: GN-04 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-04 em `../../git-notch-backlog-v0.1.md` e as seções 7, 12, 15 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src/drawer/, src/shared/, comandos tipados Rust. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Conectar lista de checkouts, grupos e arquivos ao estado Rust. Busca local, paths distinguíveis, filtro de limpos e estados indisponíveis. Implementar validação de seleção/época desde a primeira chamada assíncrona.

## Critérios obrigatórios

Mesmo arquivo aparece em dois grupos. Repo removido não afeta outros. Resposta antiga de lista não muda seleção atual. Conteúdo de nomes é texto inerte.

## Teste do avaliador

Navegar fixture com nomes iguais e alternar grupos rapidamente sob atraso artificial.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-04A.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem watchers ou painel de métricas. Não duplicar sugestões e repos.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
