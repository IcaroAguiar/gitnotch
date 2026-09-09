# GN-02C: interpretar estados git sem perder caminhos

Estado: planejado. Dependência: GN-02B aceito e incorporado à main. Origem: GN-02 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-02 em `../../git-notch-backlog-v0.1.md` e as seções 8.3, 9.4, 12, 16.1 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src-tauri/src/git/, tipos de snapshot e fixtures. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Implementar status porcelain v2 -z com paths preservados, grupos separados e IDs opacos. Tratar rename/cópia, conflitos, unborn, detached, submódulos e status incompleto.

## Critérios obrigatórios

Comparar fixtures com Git direto sob mesmas guardas. Cobrir staged e unstaged no mesmo arquivo, nomes com espaços/newlines, bytes não UTF-8 no Unix, submódulos e saída limitada que nunca aparece como limpa. Repetir prova de não mutação.

## Teste do avaliador

Rodar integração e conferir os grupos esperados de cada fixture.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-02C.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem renderer, watchers ou cálculo de patch.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
