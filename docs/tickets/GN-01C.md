# GN-01C: validar material e patch estático

Estado: planejado. Dependência: GN-01B aceito e incorporado à main. Origem: GN-01 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-01 em `../../git-notch-backlog-v0.1.md` e as seções 2, 5, 6 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src/diff/, src/drawer/, src-tauri/src/desktop/, dependências e lockfiles. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Validar window-vibrancy candidato e @pierre/diffs com patch sintético local. Carregar renderizador sob demanda. Aplicar material público Regular onde disponível, fallback sólido identificado e slider que modifica preenchimento.

## Critérios obrigatórios

Abrir build offline. Conferir texto opaco nos extremos do slider e fallback explícito. Alternar material sem acumular views nativas. Medir memória com renderizador e verificar nitidez sobre fundos claros e escuros.

## Teste do avaliador

Abrir patch estático, variar transparência e forçar sólido. Registrar screenshots reais e modo efetivo.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-01C.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem leitura de repos. Compatibilidade e reveal continuam pendentes nos SOs não testados.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
