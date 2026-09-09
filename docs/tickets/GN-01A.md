# GN-01A: criar aplicação mínima e checks

Estado: planejado. Dependência: GN-00 aceito e incorporado à main. Origem: GN-01 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-01 em `../../git-notch-backlog-v0.1.md` e as seções 3, 15, 17 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

src/, src-tauri/, manifests, lockfiles, configuração de build e CI, README, docs/decisions.md. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Criar Tauri 2, React/TypeScript e Vite estático com janela mínima. Fixar ferramentas e dependências compatíveis. Documentar comandos reais de instalação, lint, typecheck, testes e build. Configurar CI para os checks existentes e builds acessíveis, sem publicação automática.

## Critérios obrigatórios

Instalar com lockfile e compilar a partir de checkout limpo. Abrir build fora do servidor de desenvolvimento e sem rede. Confirmar que não há permissão genérica de shell no frontend. Registrar versões e plataformas realmente executadas.

## Teste do avaliador

Abrir o aplicativo empacotado e fechá-lo. Os comandos do README devem funcionar no ambiente documentado.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-01A.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem notch completo, material, renderizador, descoberta ou leitura Git.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
