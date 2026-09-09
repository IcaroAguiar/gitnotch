# GN-05A: planejar watches e limitar fila

Estado: planejado. Dependência: GN-04B aceito e incorporado à main. Origem: GN-05 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-05 em `../../git-notch-backlog-v0.1.md` e as seções 10.1, 10.2, 14 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src-tauri/src/watch.rs, scheduler e testes. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Instalar watches por diretório, metadados externos e paths rastreados necessários. Eventos invalidam status com debounce 250 ms, espera máxima 1 s, concorrência global 2 e por repo 1. Fila limitada e coalescida.

## Critérios obrigatórios

Atomic save, burst, worktrees separados e arquivo rastreado em dist geram atualização. Inspecionar watches efetivamente registrados, não só eventos filtrados. Gaveta oculta não calcula patches.

## Teste do avaliador

Editar fixture com app aberto e observar atualização. Registrar concorrência máxima e quantidade de watches.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-05A.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem alterar kernel, daemon ou consultar diffs em polling.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
