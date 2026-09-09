# GN-02D: ler comparações e estados especiais

Estado: planejado. Dependência: GN-02C aceito e incorporado à main. Origem: GN-02 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-02 em `../../git-notch-backlog-v0.1.md` e as seções 9, 12, 13, 14, 16.1 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src-tauri/src/git/, src-tauri/src/diff.rs, fixtures. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Ler staged e unstaged com parâmetros normativos e rename com ambos os paths. Ler não rastreados com limite. Tratar binário, grande, encoding, symlink, gitlink, conflito e objeto ausente.

## Critérios obrigatórios

Comparar patches ao Git direto. Testar unborn, pathspec literal, alvo externo de symlink, linha gigante, limite de patch e clone parcial sem busca. Confirmar que erro não entrega patch truncado e que os checkouts não mudam.

## Teste do avaliador

Reproduzir os dois patches diferentes do mesmo arquivo e as mensagens de limitação.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-02D.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem rede, escrita no checkout, resolução de conflito ou cache complexo.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
