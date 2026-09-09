# Contribuir com o Git Notch

Escolha um ticket no [plano de implementação](docs/tickets/README.md) e confirme seu escopo antes de implementar. Leia as seções indicadas da especificação. Proponha mudanças de produto antes de misturá-las a uma correção.

## Prepare a alteração

1. Parta da main atualizada e crie uma branch para um ticket.
2. Preserve alterações locais alheias. Faça staging por caminhos explícitos.
3. Implemente o comportamento e os testes relevantes no mesmo incremento.
4. Registre comandos reproduzíveis e resultados. Abra uma PR usando o template.
5. Aguarde revisão do mantenedor. Não habilite auto-merge por conta própria.

Ainda não existem comandos de build ou CI. GN-01A deve introduzi-los junto com a aplicação. Não adicione checks que apenas retornam sucesso sem verificar algo.

## Mantenha o repositório enxuto

Versione código, testes, fixtures sintéticas pequenas, documentação necessária e lockfiles. Coloque documentação nova em docs, templates e automações de GitHub em .github e futuros testes no local definido pela implementação.

Não versione dependências instaladas, builds, instaladores, caches, logs brutos, perfis pessoais de ferramentas, transcrições de agentes, credenciais ou cópias de projetos reais. Use .local para notas privadas e artifacts para resultados locais de testes. Esses diretórios são ignorados.

Screenshots e vídeos de revisão devem usar fixtures sintéticas e ficar como anexos da PR quando possível. Antes de anexar, confira caminhos, nomes de usuário, conteúdo de outros aplicativos e segredos. Relatórios versionados devem conter resultados resumidos e instruções reproduzíveis, sem depender de caminhos pessoais.

Não acrescente dependências ou automações sem necessidade do ticket. Versione lockfiles e confira licenças ao adicionar bibliotecas. Não copie código cuja licença ainda não foi avaliada.

## Verifique o que mudou

Use fixtures temporárias para operações Git que exigem escrita. Nunca prepare testes alterando repositórios pessoais observados pelo aplicativo. Mudanças de leitura Git precisam comprovar resultados e ausência de mutação. Mudanças de integração desktop precisam de teste nativo, não somente browser.

Registre ambientes indisponíveis como não testados. Não publique dados de performance estimados como medições. Antes do commit, confira git diff --cached e git diff --cached --check. Antes da publicação, revise também os commits que serão enviados; remover um arquivo no último commit não remove seu conteúdo do histórico.

## Relate problemas

Inclua comportamento esperado, comportamento observado, ambiente e reprodução mínima sem dados pessoais. Para possível exposição de dados ou vulnerabilidade, siga [SECURITY.md](SECURITY.md).

O projeto usa a licença MIT. Contribua somente com material que você tenha direito de disponibilizar sob essa licença.
