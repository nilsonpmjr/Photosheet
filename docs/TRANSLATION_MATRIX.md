# Matriz de Tradução e Sincronização Upstream — Photosheet
> Guia de correspondência direta arquivo a arquivo e fluxo de sincronização contínua com o repositório original `robbietilton/Compositor`.

---

## 1. Fluxo de Sincronização com o Upstream

O repositório é configurado com dois remotes:
- **`origin`:** `https://github.com/nilsonpmjr/Photosheet.git` (seu fork com a versão nativa Linux)
- **`upstream`:** `https://github.com/robbietilton/Compositor.git` (repositório original macOS)

### Como receber novas funcionalidades do original sem quebrar o Linux:
1. Buscar as atualizações do autor original:
   ```bash
   git fetch upstream main
   ```
2. Inspecionar o que mudou no upstream:
   ```bash
   git log HEAD..upstream/main --oneline
   ```
3. **Kernels C (`Compositor/Rendering/*.c`):**
   Como os arquivos C permanecem em seus caminhos originais e o `build.rs` do Rust compila diretamente de lá, melhorias de algoritmo em C são incorporadas diretamente com `git merge upstream/main` sem conflito!
4. **Funcionalidades em Swift (`Compositor/Document/`, `Compositor/UI/`, etc.):**
   Os arquivos Swift originais servem como **especificação viva de referência**. Ao verificar um diff no upstream (`git diff HEAD..upstream/main Compositor/Document/Curves.swift`), a alteração correspondente é traduzida para o arquivo equivalente em `src/core/curves.rs`.

---

## 2. Mapeamento Arquivo a Arquivo (Swift -> Rust/GTK4/Vulkan)

### 2.1 Núcleo de Processamento C (Reaproveitamento Direto 1:1 via FFI)
| Arquivo Original em `Compositor/Rendering/` | Status no Photosheet | Papel |
| :--- | :--- | :--- |
| `AdjustPixels.c` / `.h` | **100% Preservado** | Ajustes rápidos de cor, brilho, contraste e curvas |
| `BrushPixels.c` / `.h` | **100% Preservado** | Rasterização de carimbo e traço do pincel |
| `ContentFill.c` / `.h` | **100% Preservado** | Algoritmo de preenchimento sensível ao conteúdo |
| `DitherPixels.c` / `.h` | **100% Preservado** | Pontilhamento de cor / dithering de alta qualidade |
| `HealPixels.c` / `.h` | **100% Preservado** | Pincel de recuperação (Spot Healing Brush) |
| `LensPixels.c` / `.h` | **100% Preservado** | Correção geométrica e distorção de lentes |
| `LevelsPixels.c` / `.h` | **100% Preservado** | Mapeamento de níveis e histograma por canal |
| `NoisePixels.c` / `.h` | **100% Preservado** | Geração determinística de ruído procedural |
| `WandPixels.c` / `.h` | **100% Preservado** | Flood-fill e algoritmo de tolerância da Varinha Mágica |

---

### 2.2 Documento, Camadas e Sessão de Edição (`Document/` -> `src/core/`)
| Arquivo Original Swift | Destino em Rust (`src/core/`) | Responsabilidade |
| :--- | :--- | :--- |
| `EditorSession.swift` | `src/core/session.rs` | Estado ativo da sessão, ferramenta selecionada, despachador de eventos |
| `EditorSession+Brush.swift` | `src/core/tools/brush.rs` | Lógica de interpolação e suavização do traço do pincel |
| `EditorSession+Projects.swift` | `src/core/session.rs` | Gerenciamento de múltiplos documentos abertos |
| `ProjectWorkspace.swift` | `src/core/workspace.rs` | Coordenadas do canvas, limites de visualização e viewport |
| `DocumentHistory.swift` | `src/core/history.rs` | Pilha de desfazer/refazer (Undo/Redo) com teto de memória VRAM/RAM |
| `DocumentLimits.swift` | `src/core/limits.rs` | Constantes de limites (30k px, 10k camadas, 100M pixels) |
| `LayerAppearance.swift` | `src/core/layer/appearance.rs` | Opacidade, visibilidade, modo de mesclagem e propriedades da camada |
| `LayerAdjustment.swift` | `src/core/layer/adjustment.rs` | Parâmetros de cada camada de ajuste (v7–v9) |
| `LayerEffects.swift` | `src/core/layer/effects.rs` | Estruturas de dados dos efeitos (Sombra, Traçado, Brilho, etc.) |
| `LayerGroups.swift` | `src/core/layer/groups.rs` | Lógica de pastas, herança de opacidade e aninhamento de árvore |
| `LayerMask.swift` | `src/core/layer/mask.rs` | Gerenciamento de máscaras raster de 8 bits |
| `LiveLayerMask.swift` | `src/core/layer/clipping.rs` | Máscaras de recorte dinâmicas (*clipping masks*) |
| `LayerMerge.swift` | `src/core/layer/merge.rs` | Algoritmos de mesclar para baixo, mesclar grupo e achatar |
| `LayerTransform.swift` | `src/core/layer/transform.rs` | Matriz afim não destrutiva, rotação, escala e translação |
| `LayerFlip.swift` | `src/core/layer/flip.rs` | Inversão horizontal e vertical de camada ou documento |
| `Selection.swift` | `src/core/selection/mod.rs` | Representação de seleção ativa em mapa de bits de 8 bits |
| `SelectionEdits.swift` | `src/core/selection/ops.rs` | Expandir, contrair, difusão (*feather*) e inversão de seleção |
| `SelectionClipboard.swift` | `src/core/selection/clipboard.rs` | Copiar pixels, copiar mesclado (*Copy Merged*) e colar |
| `FloatingSelection.swift` | `src/core/selection/floating.rs` | Mover pixels demarcados vs mover contorno da demarcação |
| `MagicWand.swift` | `src/core/selection/wand.rs` | Orquestração da Varinha Mágica chamando `WandPixels.c` |
| `ColorRangeSelection.swift` | `src/core/selection/color_range.rs` | Seleção por intervalo de cor com tolerância de gama |
| `ObjectSelection.swift` | `src/core/selection/object.rs` | Rastreamento de contorno de objeto |
| `SubjectRemoval.swift` | `src/core/selection/subject.rs` | Segmentação e remoção de fundo (preparado para ONNX/Rembg) |
| `GuidedMatte.swift` | `src/core/selection/matte.rs` | Refinamento de bordas de máscara |
| `MaskTracing.swift` | `src/core/selection/tracing.rs` | Conversão de contorno de máscara em seleção |
| `Curves.swift` | `src/core/adjustments/curves.rs` | Interpolação de curvas spline (composto RGB e canais individuais) |
| `Levels.swift` / `LevelsAutomatic.swift` | `src/core/adjustments/levels.rs` | Níveis de entrada/saída, gama e cálculo de Auto Levels |
| `HueSaturation.swift` | `src/core/adjustments/hue_sat.rs` | Rotação de matiz e saturação com modo colorir |
| `CameraRaw*.swift` | `src/core/adjustments/camera_raw.rs` | Pipeline de desenvolvimento RAW (Luz, Cor, Curvas, Mixer, Geometria) |
| `Filters.swift` | `src/core/adjustments/filters.rs` | Filtros espaciais (Desfoques, Nitidez, Ruído, Vinheta, Bloom) |
| `TypeTool.swift` | `src/core/tools/type_tool.rs` | Lógica de texto, parágrafos, quebra de linha e runs tipográficos |
| `ShapeTool.swift` | `src/core/tools/shape.rs` | Retângulos, elipses, linhas e cantos arredondados paramétricos |
| `Crop.swift` | `src/core/tools/crop.rs` | Retângulo de corte com proporções e ancoragem |
| `CloneStamp.swift` | `src/core/tools/clone.rs` | Ponto de origem de amostragem e projeção de carimbo |
| `BlurTool.swift` | `src/core/tools/blur_tool.rs` | Aplicação localizada de desfoque por pincel |
| `SmudgeLiquify.swift` | `src/core/tools/smudge.rs` | Deformação e deslocamento de pixels |
| `Guides.swift` | `src/core/guides.rs` | Guias de alinhamento e regras de snapping magnético |

---

### 2.3 Persistência, I/O e Formatos (`IO/` -> `src/io/`)
| Arquivo Original Swift | Destino em Rust (`src/io/`) | Responsabilidade |
| :--- | :--- | :--- |
| `ProjectStore.swift` | `src/io/project_store.rs` | Gravação e leitura atômica do pacote `.comp` (v1–v11) |
| `ProjectDigest.swift` | `src/io/digest.rs` | Cálculo de hash SHA-256 para integridade e cache de assets |
| `ProjectWatcher.swift` | `src/io/watcher.rs` | Monitoramento inotify para atualização ao vivo do `.comp` |
| `ImageImporter.swift` | `src/io/importer.rs` | Decodificação de PNG, JPEG, TIFF, SVG |
| `ImageExporter.swift` | `src/io/exporter.rs` | Codificação de PNG e JPEG com controle de qualidade |
| `RawImporter.swift` | `src/io/raw.rs` | Leitura de imagens Camera RAW |
| `PSD/PSDReader.swift` | `src/io/psd/reader.rs` | Parser de cabeçalho, recursos e camadas de PSD/PSB |
| `PSD/PSDChannelCoder.swift` | `src/io/psd/channel.rs` | Descompressão RLE PackBits e decodificação de canais |
| `PSD/PSDDocumentBuilder.swift` | `src/io/psd/builder.rs` | Conversão da árvore PSD em árvore de camadas Photosheet |
| `PSD/PSDVector.swift` | `src/io/psd/vector.rs` | Caminhos e formas vetoriais do Photoshop |
| `PSD/PSDText.swift` | `src/io/psd/text.rs` | Extração de registros de texto do Photoshop |

---

### 2.4 Renderização e GPU (`Rendering/` -> `src/render/`)
| Arquivo Original Swift / Metal | Destino em Rust / WGSL (`src/render/`) | Substituição Tecnológica |
| :--- | :--- | :--- |
| `GPUCanvas.swift` | `src/render/vulkan_canvas.rs` | Inicialização `wgpu` Vulkan no lugar de `MTLDevice` |
| `LayerRenderer.swift` | `src/render/compositor.rs` | Passos de renderização de árvore de camadas |
| `TiledLayerRenderer.swift` | `src/render/tiling.rs` | Particionamento em tiles para grandes imagens |
| `MetalLayerEffects.swift` | `src/render/effects.rs` | Efeitos de sombra/stroke em shaders Vulkan (WGSL/SPIR-V) |
| `MetalBrushCoverage.swift` | `src/render/brush_gpu.rs` | Cálculo de cobertura de traço acelerado por GPU |
| `MetalWarp.swift` | `src/render/warp_gpu.rs` | Shaders de deformação livre e distorção |
| `SeparableBlend.swift` | `src/render/blend_gpu.rs` | Shaders dos 27 blend modes executados em GPU |
| `DownsampleCache.swift` | `src/render/downsample.rs` | Geração de mipmaps / pirâmides de redução |
| `CanvasViewport.swift` | `src/render/viewport.rs` | Matriz de projeção do canvas (pan, zoom, rotação) |
| `EditorCanvas.swift` | `src/render/canvas_presenter.rs` | Pipeline de apresentação final do canvas |

---

### 2.5 Interface Gráfica (`UI/` -> `src/ui/`)
| Arquivo Original SwiftUI/AppKit | Destino em GTK4 / Libadwaita (`src/ui/`) | Componentes Linux Utilizados |
| :--- | :--- | :--- |
| `CompositorApp.swift` | `src/app.rs` | `adw::Application` |
| `ContentView.swift` | `src/ui/window.rs` | `adw::ApplicationWindow` + Layout Paned |
| `ProjectTabs.swift` | `src/ui/tabs.rs` | `adw::TabBar` + `adw::TabView` |
| `LayersPanel.swift` | `src/ui/layers_panel.rs` | `gtk::ListView` com drag-and-drop nativo |
| `NativeLayerList.swift` | `src/ui/layer_row.rs` | Widget customizado da linha de camada |
| `BlendModePicker.swift` | `src/ui/widgets/blend_picker.rs` | `gtk::DropDown` com grupos estilizados |
| `BrushControls.swift` | `src/ui/tool_bars/brush_bar.rs` | Barra de controles contextuais (sliders + presets) |
| `NumericScrub.swift` | `src/ui/widgets/numeric_scrub.rs` | Widget de arrasto horizontal sobre label |
| `CanvasRulers.swift` | `src/ui/widgets/rulers.rs` | Réguas desenhadas com Cairo/GDK sobre a borda |
| `CurvesControls.swift` | `src/ui/dialogs/curves.rs` | Gráfico interativo com nós de controle spline |
| `LevelsSheet.swift` | `src/ui/dialogs/levels.rs` | Histograma em tempo real com sliders triangulares |
| `CameraRawControls.swift` | `src/ui/dialogs/camera_raw.rs` | Painel expansível com sliders Libadwaita |
| `JPEGExportSheet.swift` | `src/ui/dialogs/export_dialog.rs` | Janela de exportação com preview de tamanho |
| `KeyboardShortcuts.swift` | `src/ui/shortcuts.rs` | `gtk::ShortcutController` global |
