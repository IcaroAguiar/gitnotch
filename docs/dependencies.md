# Dependências diretas

As versões abaixo são as resolvidas para o aplicativo. Consulte os lockfiles para a árvore transitiva. Não há dependência de serviço remoto em runtime.

| Dependência | Versão | Licença declarada | Uso |
|---|---|---|---|
| @tauri-apps/api | 2.11.1 | Apache-2.0 OR MIT | Comandos IPC e label da janela |
| React e React DOM | 19.2.8 | MIT | Interface |
| Tauri | 2.11.5 | MIT ou Apache-2.0 | Janela e runtime nativo |
| tauri-build | 2.6.3 | MIT ou Apache-2.0 | Build Rust |
| Tauri CLI | 2.11.4 | MIT ou Apache-2.0 | Ferramenta de build |
| Vite | 8.2.2 | MIT | Build frontend |
| Plugin React do Vite | 6.1.0 | MIT | Build frontend |
| TypeScript | 7.0.2 | Apache-2.0 | Verificação de tipos |
| Biome | 2.5.10 | MIT ou Apache-2.0 | Lint e formatação |
| Tipos Node | 22.20.1 | MIT | Desenvolvimento |
| Tipos React | 19.2.18 | MIT | Desenvolvimento |
| Tipos React DOM | 19.2.5 | MIT | Desenvolvimento |
| serde | 1.0.229 | MIT ou Apache-2.0 | Serialização de tipos e modelos de domínio |
| serde_json | 1.0.151 | MIT ou Apache-2.0 | Serialização de dados para IPC |
| objc2 | 0.6.4 | MIT | Ponte Rust para objetos Objective-C no adaptador macOS |
| objc2-app-kit | 0.3.2 | Zlib ou Apache-2.0 ou MIT | Janela, animação e `NSGlassEffectView` no macOS |
| objc2-foundation | 0.3.2 | MIT | Tipos geométricos usados pelo adaptador macOS |
| objc2-quartz-core | 0.3.2 | Zlib ou Apache-2.0 ou MIT | Curvas de tempo da animação AppKit |
| block2 | 0.6.2 | MIT | Callback de conclusão de `NSAnimationContext` |

As licenças foram consultadas nos metadados npm e nos manifests dos crates baixados. A lista não substitui os textos de licença nem a revisão transitiva necessária para uma release distribuída. `pnpm licenses list` permite inspecionar a árvore JavaScript instalada.
