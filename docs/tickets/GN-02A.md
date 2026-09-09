# GN-02A: executar git com ambiente e recursos limitados

Estado: planejado. Dependência: GN-01C aceito e incorporado à main. Origem: GN-02 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-02 em `../../git-notch-backlog-v0.1.md` e as seções 9.1, 9.2, 14, 16.1 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src-tauri/src/git/, infraestrutura de fixtures temporárias. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Localizar Git e validar capacidades. Criar executor privado com argumentos separados, ambiente controlado, guardas, stdout/stderr drenados, timeout, cancelamento e coleta do processo. Inicialmente testar comandos inofensivos e processos sintéticos.

## Critérios obrigatórios

Testar Git ausente/incompatível, ambiente Git herdado, stderr volumoso, timeout, excesso de saída e cancelamento sem processo órfão. Nenhum IPC aceita comando arbitrário.

## Teste do avaliador

Rodar testes com saída visível e confirmar término dos processos de teste.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-02A.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Não consultar status/diff de worktrees antes do preflight GN-02B. Sem shell genérico ou alterações globais.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
