# GN-08A: medir e fechar regressões da v0.1

Estado: planejado. Dependência: GN-07B aceito e incorporado à main. Origem: GN-08 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-08 em `../../git-notch-backlog-v0.1.md` e as seções 14, 16 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

testes, scripts de medição, docs/validation.md. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Consolidar cenários críticos, fixture de carga e procedimentos de medição. Medir startup, descoberta, evento/lista, diff, CPU, memória com auxiliares e frames conforme seção 16.4.

## Critérios obrigatórios

Registrar valores, ferramenta, ambiente e amostras. Comparar com metas, sem inventar baseline ausente. Corrigir regressões dentro do escopo ou registrar ticket corretivo necessário antes do encerramento.

## Teste do avaliador

Reexecutar benchmark documentado e revisar fluxo multi-repo. Árvore real só com acesso autorizado e leitura.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-08A.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem otimização especulativa ou declarar metas atingidas sem medição.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
