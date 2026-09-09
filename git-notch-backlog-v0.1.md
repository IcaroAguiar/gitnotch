# Git Notch — backlog executável v0.1

Data: 09/09/2026. Documento normativo: `git-notch-spec-v0.1.md`. As fontes e os limites estão nesse documento. Este backlog não autoriza ampliar o produto.

## Regras para o agente implementador

Trabalhar em um repositório próprio do Git Notch. Não modificar os projetos observados, nem criar commits, branches ou fixtures dentro deles. Testes que exigem escrita usam apenas diretórios temporários. Não instalar ferramentas globais, alterar permissões, configurar `safe.directory=*`, mudar sysctl, habilitar serviços ou usar credenciais sem instrução explícita.

Implementar um incremento vertical por vez. Rodar e registrar os testes disponíveis após cada incremento. Marcar como “não testado” qualquer ambiente não acessível; screenshots conceituais e navegador local não provam integração nativa. Não continuar para aprimoramentos visuais se leitura Git ou isolamento de checkouts estiver incorreto.

Persistir decisões relevantes em `docs/decisions.md`, resultados em `docs/validation.md` e versões efetivas nos lockfiles. Guardar o foco: atividade → checkout → arquivo → diff → recolher.

## GN-01 — Estrutura mínima e prova de janela/material

**Dependências:** nenhuma. **Marco:** M0.

Criar Tauri 2 com React/TypeScript, build Vite estático e Rust. Duas entradas leves: notch e drawer, estado autoritativo compartilhado no Rust. Testar `window-vibrancy` 0.8.0 com `Regular`, sem valores experimentais. Fazer fallback sólido explícito. Avaliar `tauri-nspanel` somente se o comportamento de foco exigir.

Entregar uma janela na borda com abertura/recolhimento e um patch estático de teste. Slider altera preenchimento, nunca opacidade do texto. Não implementar descoberta completa neste ticket.

**Aceite:** build de produção abre fora do dev server; aba não rouba foco; não existe região invisível grande capturando cliques; material e fallback ficam identificáveis; alternar efeito não acumula views nativas; transparência zero e máxima funcionam. Registrar memória de ambas as janelas. Se a integração nativa falhar, registrar e simplificar o efeito antes de acrescentar dependências.

**Decisão de saída:** confirmar duas janelas ou escolher uma redimensionável, sem alterar os demais módulos. Aprovar integração de material e estratégia de reveal; não prometer morph líquido completo.

## GN-02 — Executor Git de leitura e política de segurança

**Dependências:** estrutura de GN-01; pode avançar enquanto o teste visual ocorre. **Marco:** M0/M1.

Implementar localização/capacidades do Git, execução sem shell, guardas por processo, limites, timeout, cancelamento e tratamento de retorno. Criar fixtures. Parser porcelain v2 com NUL e bytes de path; diff staged/unstaged. Implementar preflight de filtros efetivamente aplicáveis e fsmonitor externo desabilitado.

**Aceite:** mesmo arquivo aparece corretamente nos dois grupos; unborn e detached não falham; caminhos com espaços e pathspec especial são literais; índice/HEAD/refs/config não mudam nas consultas; filtro externo sintético não é executado pelo app. Limite de saída mata e colhe o processo. Nenhum comando genérico de shell é exposto pelo IPC.

**Não fazer:** contornar filtros desabilitando-os e apresentar resultado como fiel; executar plugins/skills de terceiros a partir de um README.

## GN-03 — Pastas-base, descoberta e identidade

**Dependências:** GN-02. **Marco:** M1.

Seletor de raízes autorizado, preferências versionadas e descoberta progressiva/cancelável. Identificar `.git` arquivo/diretório; resolver caminhos Git privados/comuns. Deduplicar raízes sobrepostas sem confundir worktrees. Exclusões curtas, visíveis e ajustáveis; seleção explícita prevalece. Expor descoberta parcial e erros por repo.

**Aceite:** fixture inspirada na Tetra corresponde à lista esperada; dois repos com mesmo nome são distinguíveis; worktrees separados; repo aninhado continua sendo encontrado; diretório excluído pode ser autorizado explicitamente; cancelar/remove-root interrompe jobs e remove acesso. Nenhuma varredura começa fora das raízes escolhidas, exceto metadados/configs Git estritamente necessários e identificados.

**Não fazer:** indexar o computador inteiro, seguir symlinks de diretórios indiscriminadamente ou usar o `.gitignore` como fronteira universal de descoberta.

## GN-04 — Gaveta e renderização do diff

**Dependências:** GN-01, GN-02, GN-03. **Marco:** M1.

Árvore repo → grupo → arquivo; busca local simples; um renderizador pesado ativo por vez. Carregar `@pierre/diffs` dinamicamente; fornecer patch do Git, modo unificado e fontes locais. IDs opacos de arquivo no IPC. Tratar binário, grande, conflito, encoding, limpo e indisponível explicitamente.

**Aceite:** números de linha/adições/remoções corretos; cabeçalho permanente identifica checkout e comparação; mesmo arquivo staged/unstaged usa conteúdo diferente; raiz/seleção não altera por resposta antiga; renomeação preserva os dois nomes; HTML/Markdown do repo não executam código; build funciona offline, sem CDN.

**Não fazer:** editor, Monaco, terminal, split view, comentários, stage ou resolução de conflito.

## GN-05 — Watch plan, reconciliação e filas

**Dependências:** GN-02 e GN-03. **Marco:** M2.

Usar `notify` com plano por diretório/checkout, exclusões reais de watches e observação dos metadados externos dos worktrees. Debounce 250 ms com limite de espera 1 s; concorrência global 2, por repo 1. Fila coalescida e limitada; reconciliação escalonada; backoff e fallback de polling declarado.

**Aceite:** atomic save continua sendo observado; mudanças no índice/HEAD aparecem; duas worktrees não compartilham estado; dependências excluídas não instalam watches ilimitados no Linux; arquivo rastreado em `dist` é contemplado; perda simulada de evento converge na reconciliação; hidden não calcula patches; suspensão/reabertura recupera o estado.

**Não fazer:** alterar sysctl, subir daemon, fazer polling de todos os diffs ou usar callback de filtro como prova de que uma árvore não foi monitorada.

## GN-06 — Sugestões, novidades e leitura estável

**Dependências:** GN-04 e GN-05. **Marco:** M2.

ActivityIndex distingue baseline, atividade observada e diff efetivamente atualizado. Status `M` repetido não basta: revisar fingerprint dos inputs afetados. Identidade de requisição/época/revisão; cache LRU; descarte de respostas obsoletas; seleção/âncora de leitura preservadas.

**Aceite:** alterações anteriores ao startup não recebem horário fictício; nova edição de arquivo já modificado produz sinal adequado; touch sem conteúdo não afirma diff novo; indicador não confunde repos com arquivos; ordem não muda durante leitura; atualização inviável sem salto fica pendente com aviso; arquivo limpo mantém mensagem no lugar selecionado; novidade durante leitura permanece não vista.

**Não fazer:** LLM, atribuição a agente, histórico de tarefas ou afirmar que alterações foram “aprovadas”.

## GN-07 — Transparência, movimento e acessibilidade

**Dependências:** GN-01, GN-04 e GN-06. **Marco:** M3.

Aplicar o contrato de três camadas da spec. Persistência local do slider com debounce, toggle de proteção de leitura, detecção nativa de redução de transparência/movimento. Estado cancelável `closed/opening/open/closing`, foco por intenção e fechamento protegido. Calibrar dimensões e transições em desktop real.

**Aceite:** texto não fica translúcido; código opaco por padrão; falha de efeito vira fallback legível; Preferências do sistema sobrepõem o slider; reduzir movimento elimina transformações; cliques rápidos não deixam janelas inconsistentes; nenhum loop de animação em repouso; menus/seleção não provocam fechamento acidental; 100 ciclos não apresentam crescimento contínuo de recursos.

**Não fazer:** shaders, captura de tela, blur gigante aninhado, animação de resize por IPC a cada frame ou ampliar escopo para reproduzir um morph idêntico em todos os SOs.

## GN-08 — Medição, pacotes e encerramento da v0.1

**Dependências:** anteriores. **Marco:** M3.

Matriz de testes automatizados + testes nativos disponíveis. Medir inicialização, descoberta, evento→lista, diff, memória com auxiliares, CPU ociosa, frames e tamanho do pacote. Produzir pacotes apenas para ambientes efetivamente validados; demais builds identificados como não validados. Registrar dependências WebView/Git e fallbacks visuais. Incluir licenças/NOTICE.

**Aceite:** testes críticos passam; comparação com a árvore real é somente leitura; métricas têm cenário e ferramenta; nenhum número de performance é inferido do tamanho do binário ou dev server; README explica Git necessário, privacidade, limites e como sair do app. Linux Wayland possui acesso funcional pela janela fallback. Auto-update e distribuição pública ficam fora do ticket.

**Critério de encerramento:** todos os requisitos da spec suportados ou explicitamente limitados, sem mutação dos repos, e uso real suficiente para revisar tarefa multi-repo sem alternar de projeto na ADE. Backlog posterior não bloqueia concluir a ferramenta pequena.

## Registro mínimo por ticket

```text
Ticket:
Commit / versão:
Comportamento implementado:
Testes executados e ambiente:
Resultados observados:
Limitações / não testado:
Métricas, se aplicável:
Dependências alteradas e licença:
Próximo ticket desbloqueado:
```

## Portões de decisão

| Portão | Avançar somente quando |
|---|---|
| Janela/material | Foco, hit-testing, texto e fallback funcionam num build real. |
| GitReader | Estados são corretos e consultas não mutam as fixtures. |
| Atividade | Nova edição sem mudança do código de status é percebida; eventos perdidos convergem. |
| Release | Métricas e compatibilidade são observadas, não presumidas. |

**Ordem recomendada:** GN-01/GN-02 → GN-03 → GN-04 → GN-05 → GN-06 → GN-07 → GN-08. Não iniciar distribuição, automações ou detalhes de identidade visual antes de fechar o vertical slice.
