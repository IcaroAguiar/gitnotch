# GN-04B: mostrar o diff real sob demanda

Estado: planejado. Dependência: GN-04A aceito e incorporado à main. Origem: GN-04 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-04 em `../../git-notch-backlog-v0.1.md` e as seções 7, 9.4, 12, 13, 15 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src/diff/, DesktopBridge, comandos de diff. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Conectar o renderer estático ao DiffService com IDs opacos e comparação explícita. Manter um renderer pesado, cabeçalho permanente e estados especiais. Descartar resultados que não correspondam à seleção/época/revisão.

## Critérios obrigatórios

Testar troca rápida entre repos/grupos com resposta lenta. Validar linhas e conteúdo staged/unstaged, rename, binário, conflito, grande, limpo e indisponível. Build offline não busca fontes ou scripts remotos.

## Teste do avaliador

Abrir checkout da fixture no build e ler diferenças reais nos dois grupos. Fazer refresh manual e preservar a seleção.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-04B.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Primeiro fluxo completo manual. Sem atividade automática ou indicação fictícia de novidade.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
