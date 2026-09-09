# GN-07A: aplicar preferências e acessibilidade

Estado: planejado. Dependência: GN-06B aceito e incorporado à main. Origem: GN-07 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-07 em `../../git-notch-backlog-v0.1.md` e as seções 5, 16.3 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src/settings/, estilos, settings.rs, desktop/. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Persistir slider com debounce e gravação final. Tema, material, proteção do código e preferências nativas de contraste/transparência/movimento. Foco visível e estados além de cor.

## Critérios obrigatórios

Preferência do SO prevalece em runtime. Texto fica opaco, código protegido por padrão, fallback sólido legível. Zoom e seleção/cópia funcionam.

## Teste do avaliador

Testar IDE clara/escura atrás, conteúdo em movimento e ajustes do SO no build.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-07A.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Não usar opacity global, captura de tela ou shader de blur.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
