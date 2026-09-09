# GN-01B: validar aba e gaveta nativas

Estado: planejado. Dependência: GN-01A aceito e incorporado à main. Origem: GN-01 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-01 em `../../git-notch-backlog-v0.1.md` e as seções 4, 6 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src/notch/, src/drawer/, src-tauri/src/desktop/, configuração de janelas. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Criar notch e drawer sólidos com estado compartilhado Rust. Abrir por clique, recolher e reutilizar drawer. Testar foco e hit-testing antes de decidir duas janelas ou uma redimensionável.

## Critérios obrigatórios

No build nativo, digitar em outro app enquanto a aba está recolhida. Clicar fora da aba e confirmar entrega do clique ao app de trás. Abrir e fechar repetidamente. Medir memória incluindo auxiliares e registrar a decisão de janela.

## Teste do avaliador

Gravar abertura, fechamento e continuidade da digitação. Mostrar as áreas reais clicáveis.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-01B.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem blur, descoberta, Git ou morph. NSPanel só se houver falha de foco reproduzida e decisão registrada.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
