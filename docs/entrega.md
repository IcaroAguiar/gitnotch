# Delegar e avaliar uma entrega

Os documentos originais permanecem normativos para o produto. O [índice de tickets](tickets/README.md) divide o backlog para execução sequencial. Os IDs são locais, não números de issues ou PRs já publicadas.

## Comece pelo Git

Delegue [GN-00](tickets/GN-00.md). O primeiro commit preserva os documentos na `main`. A branch seguinte contém as regras e este plano, formando a primeira PR revisável. Sem um commit de base, não há comparação útil para essa primeira PR.

Destino, visibilidade e licença ainda não foram escolhidos. O agente pode preparar o Git local e deve pedir somente os dados necessários para a publicação quando chegar a esse passo.

## Entregue um ticket ao agente

Copie este texto e substitua apenas o ID e o caminho do ticket.

```text
Implemente somente <ID> do Git Notch conforme <caminho do ticket>.
Leia AGENTS.md e as seções da especificação indicadas no ticket.
Confira que as dependências foram incorporadas à main.
Trabalhe em uma branch própria sobre a main, preservando mudanças alheias.
Antes de editar, resuma o escopo, arquivos previstos e verificações.
Implemente os critérios deste ticket e registre evidências reproduzíveis.
Não amplie o produto nem implemente o próximo ticket.
Abra uma PR no remoto já autorizado, sem merge ou auto-merge.
Se não houver remoto autorizado, entregue o diff local e informe a pendência.
Informe SHA, comandos executados, resultados e roteiro para eu testar.
Marque explicitamente testes nativos ou plataformas não verificados.
Pare após entregar esta PR para minha avaliação.
```

## Avalie antes de avançar

1. Confira o ticket e o diff completo. Investigue arquivos fora do escopo.
2. Execute os comandos registrados no SHA entregue. Confira resultado observado e esperado.
3. Para mudanças de interface, abra o build indicado e repita o cenário nativo. Para leitura Git, use a fixture descartável indicada pelo agente.
4. Solicite correções na mesma PR. Um novo commit exige rever as evidências afetadas.
5. Faça o merge somente após aceitar os critérios. Delegue o próximo ticket com a `main` atualizada.

Os primeiros tickets de backend são avaliados por testes de integração com repositórios temporários. Não precisam criar uma UI provisória só para demonstrar leitura Git. O primeiro fluxo completo com checkout real aparece em GN-04B. Mudanças de foco e material já são testáveis em GN-01B e GN-01C.

## Registre resultados

Cada ticket cria `docs/validation/<ID>.md` com comportamento, ambiente, comandos, resultados, limitações e próximos critérios desbloqueados. O registro associa a execução ao commit de código testado. Se um commit posterior muda apenas o próprio relatório, declare isso sem fingir que o SHA final foi retestado.

Mantenha `docs/validation.md` como índice desses registros quando GN-00 o introduzir. GN-01A cria `docs/decisions.md`. Estados sugeridos para tickets são planejado, em execução, em avaliação, precisa de correção e concluído. Concluído exige aceite e merge, não apenas resposta do agente.

Não imponha dez agentes, uma infraestrutura de coordenação ou metas de performance artificiais a PRs de documentação. Use as metas da seção 16.4 quando o comportamento correspondente existir. Dados ainda não medidos permanecem pendentes.
