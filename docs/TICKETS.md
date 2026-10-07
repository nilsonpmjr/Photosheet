# Plano de Tarefas e Implementação — Photoslop (/to-tickets)
> Decomposição do projeto em épicos e tickets acionáveis, com escopo claro, critérios de aceitação e arquivos envolvidos para a construção do **Photoslop** (Rust + GTK4 + Vulkan).

---

## Índice das Fases de Desenvolvimento

- [Fase 1: Fundação do Projeto, Workspace Cargo e Kernels C](#fase-1-fundação-do-projeto-workspace-cargo-e-kernels-c)
- [Fase 2: Modelo de Domínio (Core Document) e I/O (.comp)](#fase-2-modelo-de-domínio-core-document-e-io-comp)
- [Fase 3: Motor Gráfico Vulkan (Render Engine)](#fase-3-motor-gráfico-vulkan-render-engine)
- [Fase 4: Shell da Interface em GTK4 e Libadwaita](#fase-4-shell-da-interface-em-gtk4-e-libadwaita)
- [Fase 5: Canvas Interativo e Ferramentas de Edição](#fase-5-canvas-interativo-e-ferramentas-de-edição)
- [Fase 6: Formatos Avançados, Polimento e Empacotamento](#fase-6-formatos-avançados-polimento-e-empacotamento)

---

## Fase 1: Fundação do Projeto, Workspace Cargo e Kernels C

### [SLOP-01] Inicialização do Projeto Cargo e Configuração de Dependências
- **Objetivo:** Inicializar o repositório Cargo em `/home/nilsonpmjr/Projects/Photoslop` configurando as dependências essenciais do ecossistema Rust no Linux.
- **Arquivos:** `Cargo.toml`, `src/main.rs`.
- **Dependências Chave:**
  - `gtk4` (v0.9+), `libadwaita` (v0.7+)
  - `wgpu` (v23+ com features Vulkan habilitadas)
  - `serde`, `serde_json`, `image`, `uuid`, `bytemuck`, `crossbeam-channel`, `cc`.
- **Critérios de Aceitação:**
  - `cargo check` e `cargo build` executam com sucesso sem warnings de dependências ausentes.

---

### [SLOP-02] Importação e Compilação dos Kernels C do Compositor
- **Objetivo:** Copiar os 9 arquivos C e seus respectivos cabeçalhos `.h` do Compositor original e configurar o `build.rs` para compilá-los nativamente via `cc-rs`.
- **Arquivos:** `c_kernels/*`, `build.rs`.
- **Arquivos C a Importar:**
  - `AdjustPixels.c`, `BrushPixels.c`, `ContentFill.c`, `DitherPixels.c`, `HealPixels.c`, `LensPixels.c`, `LevelsPixels.c`, `NoisePixels.c`, `WandPixels.c`.
- **Critérios de Aceitação:**
  - O script de compilação gera a biblioteca estática nativa `libckernels.a` durante a execução do `cargo build`.

---

### [SLOP-03] Camada FFI Segura em Rust para os Kernels C
- **Objetivo:** Criar um módulo de interoperabilidade segura com declarações `extern "C"` e funções idiomáticas em Rust para invocar os algoritmos de processamento de pixel.
- **Arquivos:** `src/ffi/mod.rs`, `src/ffi/c_bindings.rs`, testes unitários.
- **Critérios de Aceitação:**
  - Testes unitários validam a invocação de `WandPixels` (preenchimento por tolerância) e `AdjustPixels` (ajustes de níveis e curvas) sobre buffers de teste RGBA.

---

## Fase 2: Modelo de Domínio (Core Document) e I/O (.comp)

### [SLOP-04] Estrutura de Documento, Camadas e Blend Modes
- **Objetivo:** Definir as estruturas de dados fundamentais do documento, árvore de camadas, pastas e modos de mesclagem.
- **Arquivos:** `src/core/document.rs`, `src/core/layer.rs`, `src/core/blend.rs`.
- **Funcionalidades:**
  - Representação de camadas (Pixel, Pasta/Grupo, Ajuste, Texto, Forma).
  - Enum `BlendMode` com os 27 modos do Photoshop/Compositor.
  - Hierarquia de grupos com aninhamento e herança de opacidade/visibilidade.
- **Critérios de Aceitação:**
  - Testes unitários validam a adição, remoção, reordenação e agrupamento de camadas.

---

### [SLOP-05] Sistema de Máscaras e Modos de Recorte (Clipping Masks)
- **Objetivo:** Implementar o gerenciamento de máscaras raster de 8 bits, máscaras de grupo e enlaces de recorte (*clipping mask*).
- **Arquivos:** `src/core/mask.rs`.
- **Critérios de Aceitação:**
  - Resolução correta da cobertura alfa resultante ao combinar camada base, máscara direta e camada de recorte.

---

### [SLOP-06] Leitor e Gravador Atômico do Formato `.comp` (v1–v11)
- **Objetivo:** Implementar compatibilidade total de I/O com projetos `.comp` do Compositor macOS.
- **Arquivos:** `src/io/manifest.rs`, `src/io/comp_format.rs`.
- **Funcionalidades:**
  - Desserialização de `manifest.json` com suporte a todas as versões de 1 a 11.
  - Salvamento atômico (escrita em diretório temporário + rename atômico).
  - Leitura e escrita dos arquivos de imagem PNG em `images/<uuid>.png`.
- **Critérios de Aceitação:**
  - Capacidade de abrir um projeto `.comp` existente e salvar novamente mantendo integridade estrutural verificável por diff de JSON.

---

### [SLOP-07] Sistema de Histórico (Undo / Redo com Snapshots)
- **Objetivo:** Implementar a pilha de histórico com suporte a Desfazer (Ctrl+Z) e Refazer (Ctrl+Shift+Z).
- **Arquivos:** `src/core/history.rs`.
- **Critérios de Aceitação:**
  - Histórico mantém limites de memória, desalocando snapshots antigos quando ultrapassar o orçamento configurado.

---

## Fase 3: Motor Gráfico Vulkan (Render Engine)

### [SLOP-08] Inicialização do Contexto Vulkan com Suporte a AMD e NVIDIA
- **Objetivo:** Configurar o pipeline Vulkan utilizando `wgpu` para seleção e inicialização de adaptadores gráficos dedicados ou integrados no Linux.
- **Arquivos:** `src/render/mod.rs`, `src/render/vulkan_context.rs`.
- **Critérios de Aceitação:**
  - Detecção e inicialização correta em GPUs AMD (Mesa RADV) e NVIDIA (driver oficial), com fallback gracioso para CPU Lavapipe.

---

### [SLOP-09] Shaders de Composição de Camadas e Modos de Mesclagem
- **Objetivo:** Escrever shaders de fragmento e computação (WGSL/SPIR-V) para mesclar camadas em tempo real em espaço sRGB linear.
- **Arquivos:** `src/render/pipeline.rs`, `src/render/shaders/composite.wgsl`.
- **Critérios de Aceitação:**
  - Renderização precisa de composições multi-camada com opacidade e blend modes acelerada por GPU.

---

### [SLOP-10] Shaders de Efeitos de Camada (Layer Effects)
- **Objetivo:** Implementar os shaders para os efeitos visuais em tempo real: Sombra Projetada, Traçado, Sombra Interna, Sobreposição de Cor e Brilho.
- **Arquivos:** `src/render/effects.rs`, `src/render/shaders/effects.wgsl`.
- **Critérios de Aceitação:**
  - Aplicação dos efeitos no canvas em tempo real sem rasterização destrutiva da camada.

---

### [SLOP-11] Cache de Texturas e Downsampling do Viewport
- **Objetivo:** Implementar pool de texturas Vulkan para gerenciar tiles de camadas e downsampling de alta qualidade ao afastar o zoom.
- **Arquivos:** `src/render/texture_cache.rs`.
- **Critérios de Aceitação:**
  - Ausência de artefatos de moiré em zoom reduzido e reaproveitamento eficiente de texturas VRAM.

---

## Fase 4: Shell da Interface em GTK4 e Libadwaita

### [SLOP-12] Janela Principal da Aplicação e Sistema de Abas
- **Objetivo:** Criar a janela principal (`adw::ApplicationWindow`) com barra de cabeçalho moderna (`adw::HeaderBar`) e suporte a múltiplos projetos em abas (`adw::TabBar`).
- **Arquivos:** `src/ui/window.rs`, `src/ui/header_bar.rs`, `src/app.rs`.
- **Critérios de Aceitação:**
  - Abertura de novos documentos em abas separadas, fechamento de abas e integração com o tema do sistema (claro/escuro).

---

### [SLOP-13] Painel Lateral de Camadas (Layers Panel)
- **Objetivo:** Criar o painel visual de gerenciamento de camadas na lateral direita da janela.
- **Arquivos:** `src/ui/layers_panel.rs`.
- **Funcionalidades:**
  - Lista de camadas em árvore (TreeModel / ListView).
  - Sliders de Opacidade, menu de Blend Modes, botões de Visibilidade (olho) e Bloqueio.
  - Miniaturas de pré-visualização da camada e de sua máscara.
  - Botões de Adicionar Camada, Pasta, Máscara, Mesclar e Deletar.
- **Critérios de Aceitação:**
  - Modificações na UI refletem imediatamente no estado do `Document` e provocam re-renderização do canvas.

---

### [SLOP-14] Barra de Ferramentas e Barra de Opções Contextuais
- **Objetivo:** Implementar a barra de ferramentas vertical à esquerda e a barra de opções da ferramenta ativa logo abaixo do cabeçalho.
- **Arquivos:** `src/ui/tool_bar.rs`, `src/ui/tool_options.rs`.
- **Funcionalidades:**
  - Ícones para: Mover (V), Seleção (M), Laço (L), Varinha (W), Corte (C), Pincel (B), Borracha (E), Carimbo (S), Recuperação (J), Gradiente (G), Texto (T), Formas (U), Conta-gotas (I), Mão (H), Zoom (Z).
  - Opções contextuais dinâmicas (tamanho do pincel, tolerância da varinha, dureza, etc.).
- **Critérios de Aceitação:**
  - Alternância de ferramentas via clique e atalhos de teclado altera os parâmetros no `EditorSession`.

---

### [SLOP-15] Painel de Ajustes de Cor e Diálogos de Filtros
- **Objetivo:** Criar painéis e caixas de diálogo para ajustes: Níveis (histograma), Curvas, Matiz/Saturação, Exposição e Desfoque.
- **Arquivos:** `src/ui/adjustments_panel.rs`, `src/ui/dialogs/levels_dialog.rs`, `src/ui/dialogs/curves_dialog.rs`.
- **Critérios de Aceitação:**
  - Pré-visualização ao vivo das alterações no canvas enquanto o usuário move os sliders.

---

## Fase 5: Canvas Interativo e Ferramentas de Edição

### [SLOP-16] Widget do Canvas GTK4 com Navegação (Pan, Zoom, Guias)
- **Objetivo:** Implementar o widget central do canvas com navegação suave, suporte a réguas e guias magnéticas.
- **Arquivos:** `src/ui/canvas_widget.rs`, `src/render/canvas_presenter.rs`.
- **Funcionalidades:**
  - Pan com Espaço+Arrastar ou botão do meio do mouse.
  - Zoom com roda do mouse (Ctrl+Scroll) ou ferramenta Zoom.
  - Réguas em pixels e criação de guias por arrasto das réguas.
  - Apresentação do frame Vulkan a 60+ FPS via buffer de textura.
- **Critérios de Aceitação:**
  - Navegação fluida sem travamentos na interface.

---

### [SLOP-17] Ferramentas de Pintura e Retoque
- **Objetivo:** Implementar os traços do Pincel, Borracha, Carimbo de Clonagem e Pincel de Recuperação (Spot Healing).
- **Arquivos:** `src/core/tools/brush.rs`, `src/core/tools/heal.rs`, `src/core/tools/clone.rs`.
- **Critérios de Aceitação:**
  - Pincel opera com suavização de traço sem latência perceptível.
  - Pincel de recuperação remove marcas com síntese de textura usando o kernel C [`HealPixels.c`].

---

### [SLOP-18] Ferramentas de Seleção e Transformação
- **Objetivo:** Implementar seleção retangular, elíptica, laço, varinha mágica e transformação de camadas.
- **Arquivos:** `src/core/selection.rs`, `src/core/tools/transform.rs`.
- **Critérios de Aceitação:**
  - Seleção exibe formigas marchantes (*marching ants*); operações de pintura e filtros respeitam a demarcação ativa.

---

### [SLOP-19] Ferramentas de Formas e Texto
- **Objetivo:** Implementar desenho de vetores geométricos e edição inline de texto tipográfico.
- **Arquivos:** `src/core/tools/shape.rs`, `src/core/tools/type_tool.rs`.
- **Critérios de Aceitação:**
  - Formas e textos permanecem editáveis até serem explicitamente rasterizados.

---

## Fase 6: Formatos Avançados, Polimento e Empacotamento

### [SLOP-20] Decodificador de Arquivos PSD/PSB
- **Objetivo:** Portar o parser de arquivos PSD/PSB do Compositor para importar arquivos do Photoshop no Linux.
- **Arquivos:** `src/io/psd_reader.rs`.
- **Critérios de Aceitação:**
  - Abertura de arquivos `.psd` com preservação de camadas, nomes, modos de mesclagem e máscaras.

---

### [SLOP-21] Janela de Exportação Interativa (JPEG / PNG)
- **Objetivo:** Criar o diálogo de exportação rápida com pré-visualização de qualidade e cálculo em tempo real do tamanho final do arquivo.
- **Arquivos:** `src/ui/dialogs/export_dialog.rs`.
- **Critérios de Aceitação:**
  - Exportação fiel mantendo resolução e metadados DPI.

---

### [SLOP-22] Empacotamento Linux (PKGBUILD Arch e Flatpak)
- **Objetivo:** Criar os manifestos de empacotamento oficial para distribuição no ecossistema Linux.
- **Arquivos:** `packaging/PKGBUILD`, `packaging/org.photoslop.Photoslop.yaml`, `data/org.photoslop.Photoslop.desktop`, ícones SVG.
- **Critérios de Aceitação:**
  - Instalação e execução com sucesso via `makepkg` no Arch Linux e validação no Flatpak Builder.
