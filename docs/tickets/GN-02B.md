# GN-02B: barrar filtros e programas externos

Estado: planejado. Dependência: GN-02A aceito e incorporado à main. Origem: GN-02 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-02 em `../../git-notch-backlog-v0.1.md` e as seções 9.2, 9.3, 15, 16.1 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src-tauri/src/git/, testes de integração Git. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Inspecionar configuração e atributos efetivos antes de leituras que possam executar filtros. Distinguir filtro configurado de aplicável. Cobrir índice/worktree e configs incluídas. Expor limitação por checkout e invalidar avaliação quando inputs mudarem.

## Critérios obrigatórios

Fixtures clean/process/fsmonitor usam marcadores e não são executadas pelo leitor. Filtro global não aplicável não bloqueia repo normal. Testar atributos no índice e worktree. Comparar hashes de arquivos, índice, HEAD, refs e configs antes/depois.

## Teste do avaliador

Executar fixtures controladas e verificar ausência dos marcadores e de mutação.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-02B.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Não desabilitar filtro silenciosamente nem declarar sandbox contra corrida maliciosa.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
