# GN-08B: gerar pacotes e documentar suporte

Estado: planejado. Dependência: GN-08A aceito e incorporado à main. Origem: GN-08 do backlog v0.1. Entregar uma PR contra main e parar para avaliação.

## Leia

Leia `../../AGENTS.md`, `../entrega.md`, o item GN-08 em `../../git-notch-backlog-v0.1.md` e as seções 2, 16.3, 17 de `../../git-notch-spec-v0.1.md`.

## Escopo de arquivos

configuração de bundle, README, licenças/NOTICE, documentação. Esses caminhos são previstos, não arquivos já existentes. Acrescente somente os testes e registros necessários. Justifique qualquer ampliação.

## Implemente

Gerar pacotes locais para ambientes acessíveis. Documentar Git/WebView, privacidade, saída do app, fallback e matriz de suporte. Conferir licenças transitivas e decisão de licença do projeto antes da distribuição.

## Critérios obrigatórios

Abrir pacote fora do ambiente dev. Testar Git ausente e acesso funcional por fallback onde disponível. Medir tamanho instalado/comprimido. Cada plataforma tem prova própria ou marca de não validada.

## Teste do avaliador

Instalar/abrir pacote local e completar pasta, repo, arquivo, diff e recolher. Conferir como sair.

O agente deve fornecer comandos reais e passos reproduzíveis no registro `docs/validation/GN-08B.md`, com pré-requisitos, resultado esperado e observado. Use fixtures temporárias exclusivas. Nunca use projetos pessoais para preparar casos que exigem escrita.

## Fora do escopo

Sem publicação pública, auto-update, credenciais de assinatura ou promessa multiplataforma sem autorização/prova.

## Entrega

Inclua os testes relevantes, evidências no SHA implementado, limitações, dependências/licenças alteradas e roteiro de revisão na PR. Critério não verificado permanece pendente. Não faça merge nem inicie o próximo ticket.
