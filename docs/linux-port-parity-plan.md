# Plano de paridade do Photosheet com o Compositor 1.4.5

Este é o plano de reescrita integral do Compositor 1.4.5 para Linux. O código
em `src/` é um ponto de partida, e não uma declaração de paridade: recursos só
são considerados concluídos quando passam pelos critérios de aceite e pelos
testes de compatibilidade definidos neste documento.

## Meta e regra de compatibilidade

O produto final é um editor Linux nativo, escrito em Rust, com GTK4/Libadwaita
e renderização por `wgpu`/Vulkan. Ele deve preservar o comportamento observável
do Compositor 1.4.5, exceto onde uma integração macOS não tem equivalente
direto no Linux (notarização, Sparkle e atalhos com Command, substituídos por
integrações Linux e Ctrl).

Uma funcionalidade está pronta somente se:

1. há teste unitário dos invariantes do domínio;
2. há teste de integração que exercita o fluxo real de arquivo ou de UI;
3. quando renderiza pixels, há imagem de referência e tolerância explícita;
4. um projeto `.comp` salvo no Photosheet abre no Compositor, e o inverso,
   sem perder campos, pixels ou hierarquia suportados;
5. a funcionalidade aparece no produto, não apenas em um módulo sem chamadas.

## Situação inicial auditada

| Área | Situação | Lacuna crítica |
| --- | --- | --- |
| modelo, blend modes e estruturas de camada | parcial | árvore, grupos, máscaras e ajustes não compõem o resultado final |
| `.comp` | parcial e inseguro | PNG é tratado como RGBA cru; troca do pacote remove o destino antes do rename |
| canvas | parcial | Cairo é o renderer efetivo; Vulkan não está ligado ao canvas |
| pintura, transform e formas | parcial | há algoritmos isolados, mas cobertura de ferramenta e integração são incompletas |
| PSD/PSB | esqueleto | cabeçalho e registros; não decodifica canais/pixels, grupos, máscaras ou texto |
| importação, RAW e exportação | incompleto | não há fluxo de abrir/salvar; exportação não usa o compositor |
| UI e atalhos | parcial | painéis básicos existem, sem DnD/hierarquia, inspector, abas reais ou atalhos completos |
| qualidade | insuficiente | 23 testes, sem fixtures reais; testes GPU em paralelo provocam `SIGSEGV` |

## Mapa de gaps por domínio

### P0 — tornar o produto correto antes de ampliá-lo

| ID | Escopo do Compositor | Estado | Entrega e critério de aceite |
| --- | --- | --- | --- |
| P0-01 | domínio de pixels, transformações e árvore de camadas | parcial | adotar imagem decodificada RGBA premultiplicada, dimensões próprias da camada e árvore validada; teste de grupos aninhados, transform e opacidade |
| P0-02 | leitura/escrita `.comp` v1–v11 | parcial | validar manifesto como o original, decodificar/validar PNG e máscaras, escrita por diretório temporário e troca sem janela de perda; corpus de projetos de cada versão |
| P0-03 | compositor de documento | ausente | compor árvore, clipping/folder masks, ajustes, efeitos, opacidade, 27 blend modes e transforms em destino transparente; golden tests por modo |
| P0-04 | apresentação Vulkan no canvas | ausente | conectar compositor, texturas e `wgpu::Surface` ao widget GTK; Cairo limitado a overlays; teste de smoke e captura de frame |
| P0-05 | fluxo de arquivos | ausente | Open, Save, Save As, importação e recuperação de erro conectados à UI; cancelar/falhar não altera o documento aberto |
| P0-06 | base de qualidade | insuficiente | `cargo fmt --check`, Clippy sem warnings novos, testes determinísticos e GPU serializada/isolada; CI Linux com Vulkan software |

### P1 — edição raster, seleção e estrutura de documento

| ID | Escopo do Compositor | Estado | Entrega e critério de aceite |
| --- | --- | --- | --- |
| P1-01 | camadas e grupos | parcial | criar, renomear, duplicar, reordenar, aninhar por DnD, bloquear, merge down/layers/group e copiar/colar entre documentos |
| P1-02 | máscaras | parcial | criar, pintar, preencher, inverter, blur, feather, aplicar, link/desvincular; clipping chains e folder masks com limite validado |
| P1-03 | transformações | parcial | mover, escala, rotação, flip, free distort, multiseleção e grupo; snapping em canvas/camada/guia/centro |
| P1-04 | seleções | parcial | marquee retangular/elíptica, lasso livre/poligonal, wand, objeto, sujeito, booleanas, expand/contract/feather, floating selection e load selection |
| P1-05 | edição e retoque | parcial | brush/erase com smoothing e Shift, clone current/all layers, spot heal, blur, smudge/liquify, content-aware fill e color range |
| P1-06 | crop e geometria do documento | ausente | crop com proporções/simetria, canvas size, image size, trim, resolução e grid/guides/rulers persistidos |

### P2 — conteúdo editável e imagem

| ID | Escopo do Compositor | Estado | Entrega e critério de aceite |
| --- | --- | --- | --- |
| P2-01 | formas e gradientes | parcial | retângulo, retângulo arredondado, elipse, linha e os gradientes linear/radial/angular/refletido/diamante permanecem editáveis |
| P2-02 | texto | parcial | edição inline, caixas e texto pontual, wrap, alinhamento, leading/tracking, fontes e `colorRuns`/`fontRuns`; rasterização apenas por ação explícita |
| P2-03 | ajustes não destrutivos | parcial | Hue/Saturation, Levels/Auto, Curves, Exposure, Gradient Map, Grain, B&W, Color Balance, Invert, Gaussian/Motion Blur e Noise na árvore de composição |
| P2-04 | filtros destrutivos | ausente | câmera RAW, vignette, bloom, tonal contrast, lens correction e remove background com preview limitado à seleção |
| P2-05 | color picker e paleta | ausente | amostragem de canvas composto, HSV/RGB/hex, anel de amostra e cores recentes |

### P3 — formatos, desempenho e experiência do produto

| ID | Escopo do Compositor | Estado | Entrega e critério de aceite |
| --- | --- | --- | --- |
| P3-01 | importação raster e SVG | ausente | JPEG/PNG/TIFF/HEIC/SVG com orientação, perfil e DPI; diálogo de importação e DnD |
| P3-02 | RAW | ausente | DNG/CR2/NEF/ARW com etapa de desenvolvimento equivalente ao Camera Raw |
| P3-03 | PSD/PSB | esqueleto | canais raw/RLE, árvores, máscaras, blend modes, formas e texto horizontal simples; relatório de conversão antes da aplicação |
| P3-04 | exportação e clipboard | parcial | PNG alfa, JPEG com qualidade/preview/tamanho, Copy Merged e metadados DPI; bytes saem do compositor real |
| P3-05 | workspace | parcial | múltiplos documentos em abas, recentes, fechar com alterações, autosave/recovery e histórico com orçamento de memória |
| P3-06 | live sync | ausente | watcher inotify para `.comp`, digest de conteúdo e recarga segura de mudanças externas |
| P3-07 | documentos grandes | ausente | tiling, cache de downsample/mipmaps, orçamento RAM/VRAM, limites e fallback previsível |
| P3-08 | atalhos e acessibilidade | parcial | mapa editável de atalhos, navegação de teclado, nomes acessíveis e comportamento GNOME consistente |
| P3-09 | distribuição Linux | parcial | instalador, Arch, `.deb` e AppImage com smoke tests em ambientes limpos |

## Ordem de execução

```text
P0: dados e I/O corretos → compositor → canvas Vulkan → fluxo de arquivos → CI
P1: camadas/máscaras → transform → seleção → pintura → geometria
P2: formas/gradientes → texto → ajustes → filtros → cor
P3: importação → PSD/PSB → exportação → workspace/live sync → escala/pacotes
```

Não iniciar PSD, Camera Raw ou novos diálogos antes de P0-01 a P0-06. Sem um
compositor único e I/O fiel, esses recursos virariam caminhos paralelos e
incompatíveis entre si.

## Plano de ações imediato

1. Criar um corpus versionado de `.comp` e PSD/PSB, com snapshots PNG de saída e
   casos inválidos. Extrair projetos reais do Compositor 1.4.5, sem depender de
   arquivos gerados pelo Photosheet.
2. Substituir `Vec<u8>` ambíguo por tipos distintos para PNG codificado, bitmap
   RGBA e máscara de 8 bits. Corrigir carregamento e escrita `.comp` e implementar
   validação equivalente ao `ProjectStore.swift`.
3. Definir uma API única `DocumentRenderer::render(document) -> Surface`; tanto
   canvas, export, picker, thumbnails e Copy Merged deverão chamá-la.
4. Ligar essa API ao `wgpu::Surface` do GTK e migrar overlays (seleção, guides,
   gizmo e cursor) para uma camada separada. Remover a composição manual do
   diálogo de exportação.
5. Instalar a disciplina de testes: um teste por bug, golden tests de pixel,
   testes de round-trip nos dois aplicativos e job de GPU serial. Só então atacar
   cada item P1–P3 na ordem estabelecida.

## Marcos de aceitação

| Marco | Demonstração exigida |
| --- | --- |
| M1 — interoperabilidade | abrir, editar e salvar corpus `.comp` v1–v11 nos dois aplicativos sem perda suportada |
| M2 — edição raster | criar composição com grupos, máscaras, blend modes, transform e seleção; exportação igual ao golden |
| M3 — conteúdo não destrutivo | formas, texto, ajustes e efeitos continuam editáveis após reabrir |
| M4 — formatos | importar raster/RAW e PSD/PSB com relatório; exportar PNG/JPEG/clipboard fiel |
| M5 — release | documento grande, recovery, live sync, atalhos, acessibilidade e pacotes Linux aprovados em smoke tests |

## Skills encontradas para apoiar a execução

Não instalei nenhuma automaticamente. As três recomendadas têm adoção alta e
complementam o trabalho, sem substituir a especificação do Compositor:

- [TDD, mattpocock/skills@tdd](https://skills.sh/mattpocock/skills/tdd) — 1 milhão de instalações; usar para o corpus e testes por paridade;
- [Improve Codebase Architecture, mattpocock/skills@improve-codebase-architecture](https://skills.sh/mattpocock/skills/improve-codebase-architecture) — 1,1 milhão; usar na reorganização P0;
- [Code Review, mattpocock/skills@code-review](https://skills.sh/mattpocock/skills/code-review) — 679,4 mil; usar como gate dos marcos.

As buscas por Rust/GTK e Vulkan/wgpu não retornaram uma skill suficientemente
madura e específica: a melhor opção WebGPU tinha 816 instalações. Por isso,
essas frentes ficam cobertas por documentação oficial e pelos testes de paridade.
