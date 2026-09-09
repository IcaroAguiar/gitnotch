# GN-07B: concluir interação e geometria de desktop

Estado: planejado. Dependência: GN-07A aceito e incorporado à main. Origem: GN-07 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-07 em `../../git-notch-backlog-v0.1.md` e as seções 4, 6, 16.3 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

desktop/, notch/, drawer/. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Concluir máquina cancelável de abertura/fechamento, Esc hierárquico, manter aberta, fechamento protegido, atalho configurável e erro de registro. Persistir posição proporcional e ajustar monitores/DPI.

## Critérios obrigatórios

Cliques rápidos não deixam janelas inconsistentes. Menu/seleção/arraste impedem fechamento acidental. Mudanças externas não roubam foco. Desconectar monitor recupera janela. Medir recursos em 100 ciclos e ausência de animação ociosa.

## Teste do avaliador

Gravar cenário real com atalhos, seleção, monitores e movimento reduzido. Declarar cenários indisponíveis.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-07B.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem IPC por frame ou paridade absoluta no Wayland.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
