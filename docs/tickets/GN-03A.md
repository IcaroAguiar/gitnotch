# GN-03A: autorizar raízes e persistir preferências

Estado: planejado. Dependência: GN-02D aceito e incorporado à main. Origem: GN-03 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-03 em `../../git-notch-backlog-v0.1.md` e as seções 3, 8.1, 12, 15 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src/settings/, src-tauri/src/settings.rs, commands.rs, app_state.rs. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Adicionar seletor nativo de pastas, schema versionado e escrita atômica das preferências próprias. Canonicalizar raízes, manter época de autorização e revogar acesso ao remover raiz.

## Critérios obrigatórios

Testar restauração, arquivo inválido, raiz removida e tentativa IPC com path arbitrário. Seleção de diálogo autoriza a raiz. Remoção invalida handles e requisições pendentes.

## Teste do avaliador

Selecionar raiz temporária, reiniciar e remover. Demonstrar erro de acesso após remoção.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-03A.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem varredura geral ou escrita nos projetos selecionados.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
