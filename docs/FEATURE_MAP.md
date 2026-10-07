# Mapa Completo de Funcionalidades — Photoslop
> Mapeamento exaustivo de todas as funcionalidades, ferramentas, filtros e formatos do **Compositor (v1.4.5)** a serem traduzidos para a arquitetura nativa Linux em **Rust + GTK4 + Vulkan**.

---

## 1. Gestão de Documentos e Formato de Arquivo

### 1.1 Formato Nativo de Projeto (`.comp`)
- **Especificação:** Pacote de diretório contendo `manifest.json` e pasta `images/` com `<layer-uuid>.png` e `<layer-uuid>.mask.png`.
- **Compatibilidade de Versões:**
  - **v1–v3:** Camadas base, resolução (DPI 1–9600), opacidade (0.0–1.0), modos de mesclagem (blend modes).
  - **v4:** Máscaras de camada em escala de cinza de 8 bits (`.mask.png`) com flag `maskEnabled`.
  - **v5:** Máscaras de recorte (*Clipping Masks*) através de `maskSourceID` com cadeias de até 256 nós.
  - **v6:** Máscaras aplicadas a pastas/grupos (*Folder Masks*).
  - **v7:** Camadas de ajuste não destrutivas (`Hue/Saturation`, `Levels`, `Curves`, `Exposure`, `Gradient Map`, `Grain`, `Invert`, `Black & White`, `Color Balance`).
  - **v8:** Opacidade independente em pastas e sistema de réguas/guias (`guides`: horizontal/vertical).
  - **v9:** Ajustes espaciais avançados (`Gaussian Blur`, `Motion Blur`, `Add Noise` com semente determinística).
  - **v10:** Camadas de texto com formatação mista por intervalos de cor (`colorRuns`).
  - **v11:** Camadas de texto com formatação mista por intervalos de fontes tipográficas (`fontRuns`).
- **Campos Aditivos:**
  - `maskPlacement` e `maskLinked`: desacoplamento de transformação entre camada e sua máscara.
  - `shape`: metadados de vetor primitivo (retângulo, elipse, linha, raio de cantos).
  - `effects`: metadados de estilos de camada não destrutivos.
- **Limites do Documento:** Até 30.000 px por lado, 100 milhões de pixels de imagem, 100 milhões de pixels de máscara, até 10.000 camadas, manifesto de até 4 MiB.
- **Gravação Atômica:** Substituição segura de diretório para evitar corrupção em caso de queda de energia ou interrupção.
- **Live Sync / File Watcher:** Suporte para que agentes ou scripts externos alterem o `.comp` em disco e o canvas atualize em tempo real sem fechar o projeto.

### 1.2 Importação e Exportação
- **Importação Raster:** PNG, JPEG, TIFF, HEIC, SVG (rasterizado na importação).
- **Importação RAW:** Abertura de arquivos Camera RAW (DNG, CR2, NEF, ARW) com pré-processamento de desenvolvimento.
- **Importação PSD/PSB (Photoshop):**
  - Leitor nativo de arquivos PSD/PSB de 8 bits em RGB.
  - Preservação de hierarquia de grupos, modos de mesclagem, opacidades, máscaras de camada, retângulos/elipses de preenchimento e textos horizontais editáveis.
  - Emissão de relatório de conversão antes da abertura do arquivo.
- **Exportação:**
  - Exportação PNG (achatada com preservação de alfa).
  - Exportação JPEG com janela interativa de ajuste de qualidade e tamanho estimado em tempo real (*Save for Web* / *Export As*).
  - Opção *Copy Merged* (⌘⇧C / Ctrl+Shift+C) para área de transferência do sistema operacional.

---

## 2. Sistema de Camadas (Layers Engine)

### 2.1 Estrutura e Hierarquia
- **Tipos de Camada:**
  - Camada de Imagem / Raster (`PixelLayer`).
  - Camada de Grupo / Pasta (`GroupLayer`, pass-through com opacidade e máscara).
  - Camada de Ajuste (`AdjustmentLayer`, afeta todas as camadas inferiores na pasta).
  - Camada de Forma Vetorial (`ShapeLayer`, retângulo, retângulo arredondado, elipse, linha).
  - Camada de Texto (`TextLayer`, parágrafos, quebras automáticas, runs de fonte e cor).
- **Operações da Lista de Camadas:**
  - Reordenação hierárquica por arrastar e soltar (drag & drop) com aninhamento de até 64 níveis.
  - Duplicação de camada ou grupo (Ctrl+J ou Alt+Arrastar).
  - Renomeação inline com duplo clique.
  - Alternância de visibilidade (ícone de olho com suporte a arrastar contínuo para múltiplas camadas).
  - Bloqueio de camada (Lock pixels / Lock position).
  - Mesclagem: *Merge Down* (Ctrl+E), *Merge Layers* (seleção múltipla) e *Merge Group*.
  - Copiar e colar camadas inteiras entre diferentes abas/documentos abertos.

### 2.2 Modos de Mesclagem (Photoshop Blend Modes)
Totalidade dos modos suportados na ordem canônica da indústria:
- **Normal:** Normal, Dissolve.
- **Escurecer:** Darken, Multiply, Color Burn, Linear Burn, Darker Color.
- **Clarear:** Lighten, Screen, Color Dodge, Linear Dodge (Add), Lighter Color.
- **Contraste:** Overlay, Soft Light, Hard Light, Vivid Light, Linear Light, Pin Light, Hard Mix.
- **Comparação / Inversão:** Difference, Exclusion, Subtract, Divide.
- **Componentes HSL:** Hue, Saturation, Color, Luminosity.

### 2.3 Máscaras de Camada
- **Máscara Raster (Layer Mask):** Escala de cinza de 8 bits; branco revela, preto oculta, cinza transparência parcial.
- **Máscara de Recorte (Clipping Mask):** Uma ou mais camadas usam o canal alfa da camada base imediatamente inferior como máscara.
- **Máscara de Pasta (Group Mask):** Uma máscara aplicada a uma pasta modula todas as camadas contidas nela.
- **Edição de Máscara:** Pintar, preencher, inverter (Ctrl+I), aplicar desfoque e difusão (*feather*).
- **Link / Desvinculação:** Permite mover ou transformar a máscara independentemente da imagem associada.

### 2.4 Efeitos e Estilos de Camada (GPU Layer Effects)
Renderizados em tempo real pela GPU com edição não destrutiva:
- **Traçado (Stroke):** Tamanho de 0 a 500 px, cor sólida, opacidade, posição interna (*inside*) ou externa (*outside*).
- **Sombra Projetada (Drop Shadow):** Ângulo (-180° a 180°), distância, desfoque (raio), cor e opacidade.
- **Sombra Interna (Inner Shadow):** Ângulo, distância, desfoque, cor e opacidade.
- **Sobreposição de Cor (Color Overlay):** Cor RGB e opacidade de mesclagem.
- **Brilho Externo (Outer Glow):** Tamanho, cor e opacidade.
- **Brilho Interno (Inner Glow):** Tamanho, cor e opacidade.
- **Controle:** Ativação/desativação individual de cada efeito sem perder os parâmetros configurados.

---

## 3. Ferramentas de Transformação

- **Transformação Livre Não Destrutiva:** Movimentação, redimensionamento, rotação e inversão mantendo o buffer de imagem em resolução total original.
- **Distorção Livre (Free Distort):** Manipulação independente dos 4 vértices do retângulo delimitador com tecla modificadora.
- **Transformação em Grupo:** Transformação síncrona de múltiplas camadas selecionadas ou pastas inteiras.
- **Snapping Magnético:** Alinhamento automático às bordas e centros do canvas, bordas de outras camadas e guias de alinhamento.
- **Inspetor Numérico:** Controle preciso de coordenadas X, Y, Largura, Altura, Escala (%) e Ângulo (°), com ajuste fino via teclas direcionais e *scrubbing* por arrasto no rótulo.
- **Inversão Rápida:** Flip Horizontal e Flip Vertical (de camada individual ou do canvas completo).

---

## 4. Ferramentas de Seleção

- **Seleções Geométricas:**
  - Letreiro Retangular (*Rectangle Marquee*).
  - Letreiro Elíptico (*Ellipse Marquee*).
- **Seleções Livres:**
  - Laço Livre (*Freehand Lasso*).
  - Laço Poligonal (*Polygonal Lasso*).
- **Seleções Inteligentes:**
  - Varinha Mágica (*Magic Wand*): Seleção por similaridade de cor com tolerância regulável e modo contíguo/global (implementado via kernel em C [`WandPixels.c`](file:///home/nilsonpmjr/Downloads/Compositor-1.4.5/Compositor/Rendering/WandPixels.c)).
  - Seleção de Objeto (*Object Selection*): Identificação e rastreamento de contorno de objeto dentro de uma área demarcada.
  - Selecionar Sujeito (*Select Subject*): Segmentação com detecção de primeiro plano.
- **Operações Booleanas de Seleção:**
  - Nova seleção, Adicionar à seleção (Shift), Subtrair da seleção (Alt), Interseção (Shift+Alt).
- **Refinamento de Seleção:**
  - Expandir (*Expand*), Contrair (*Contract*), Difusão (*Feather*).
  - Inverter seleção (Ctrl+Shift+I).
  - Desfazer seleção (Ctrl+D).
- **Seleção Flutuante:**
  - Mover apenas o contorno da seleção ou mover/cortar os pixels selecionados.
  - Carregar canal alfa da camada ou máscara como seleção ativa (Ctrl+Clique na miniatura).
- **Preenchimento com Reconhecimento de Conteúdo (*Content-Aware Fill*):**
  - Síntese de pixels que preenche áreas selecionadas ou expande bordas do canvas usando análise de vizinhança (kernel C [`ContentFill.c`](file:///home/nilsonpmjr/Downloads/Compositor-1.4.5/Compositor/Rendering/ContentFill.c)).

---

## 5. Pintura, Retoque e Desenho

- **Pincel (*Brush Tool* - tecla B):**
  - Ajuste de diâmetro (tamanho), dureza (*hardness*), opacidade e fluxo (*flow*).
  - Algoritmo de suavização de traço (*stroke smoothing / stabilizer*).
  - Linhas retas com Shift+Clique.
  - Alternância rápida para modo borracha (E).
- **Pincel de Recuperação de Manchas (*Spot Healing Brush*):**
  - Remoção inteligente de imperfeições com síntese de textura baseada no entorno imediato (kernel C [`HealPixels.c`](file:///home/nilsonpmjr/Downloads/Compositor-1.4.5/Compositor/Rendering/HealPixels.c)).
- **Carimbo de Clonagem (*Clone Stamp Tool*):**
  - Definição de ponto de origem (Alt+Clique).
  - Modo alinhado ou não alinhado.
  - Amostragem: Camada atual (*Current Layer*) ou Todas as camadas (*All Layers*).
- **Ferramenta de Desfoque (*Blur Tool*):**
  - Desfoque localizado aplicado diretamente sobre pixels ou sobre máscaras.
- **Ferramenta de Gradiente (*Gradient Tool*):**
  - Tipos: Linear, Radial, Angular, Refletido e Diamante.
  - Editor de paradas de cor (*color stops*) e opacidade.
- **Ferramenta de Formas Vetoriais (*Shape Tool*):**
  - Criação de Retângulos, Retângulos com Cantos Arredondados, Elipses e Linhas.
  - Parâmetros permanecem vetoriais e editáveis até serem explicitamente rasterizados.
- **Ferramenta de Texto (*Type Tool* - tecla T):**
  - Criação de caixas de texto com parágrafo e quebra automática ou texto de ponto.
  - Edição tipográfica: Família de fontes do sistema, tamanho, cor, alinhamento (esquerda, centro, direita), tracking (espaçamento entre letras) e leading (espaçamento entre linhas).
  - Formatação heterogênea: letras individuais com cores e fontes diferentes dentro da mesma caixa de texto.
  - Suporte para uso da camada de texto como base de *Clipping Mask*.
- **Conta-gotas (*Eyedropper*) e Seletor de Cores (*Color Picker*):**
  - Amostragem de pixel no canvas com anel de visualização ampliada (*sample ring*).
  - Seletor de cores completo (HSV, RGB, hexadecimal) e paleta de cores recentes.

---

## 6. Camadas de Ajuste e Filtros de Imagem

### 6.1 Ajustes de Cor e Tom
- **Níveis (*Levels*):** Histogramas por canal (Composto RGB, Vermelho, Verde, Azul), pontos de preto, branco e gama intermediária, botão *Auto Levels* inteligente.
- **Curvas (*Curves*):** Ajuste de curvas com interpolação spline por pontos de controle (canais RGB, R, G, B).
- **Matiz/Saturação (*Hue/Saturation*):** Deslocamento de matiz (±180°), saturação (±100), luminosidade (±100) e modo colorir (*Colorize*).
- **Exposição (*Exposure*):** Exposição (EV), deslocamento de tons escuros (*Offset*) e correção de gama (*Gamma*).
- **Mapa de Gradiente (*Gradient Map*):** Mapeamento da luminância da imagem sobre um gradiente arbitrário de sombras a realces com opção de inverter.
- **Equilíbrio de Cores (*Color Balance*):** Ajustes de Ciano-Vermelho, Magenta-Verde e Amarelo-Azul divididos por Sombras (*Shadows*), Meios-tons (*Midtones*) e Realces (*Highlights*).
- **Preto e Branco (*Black & White*):** Conversão monocromática com controle de peso relativo dos canais de cor (Vermelho, Amarelo, Verde, Ciano, Azul, Magenta).
- **Inverter (*Invert*):** Inversão matemática de valores de pixel (255 - valor).
- **Granulação de Filme (*Film Grain*):** Simulação de textura física de filme fotográfico.

### 6.2 Filtros Espaciais e Efeitos
- **Desfoque Gaussiano (*Gaussian Blur*):** Raio de 0.1 a 250 px, com expansão automática além dos limites originais da camada.
- **Desfoque de Movimento (*Motion Blur*):** Ângulo de -90° a +90° e distância de 1 a 2000 px.
- **Adicionar Ruído (*Add Noise*):** Quantidade ajustável, opções de distribuição Gaussiana vs Uniforme, monocromático e semente estável.
- **Vinheta (*Vignette*):** Escurecimento/clareamento periférico com controle de raio e ponto médio.
- **Brilho Difuso / Bloom (*Bloom / Glow*):** Efeito de realce e difusão de áreas de alta luminosidade.
- **Contraste Tonal (*Tonal Contrast*):** Realce de microcontraste e textura sem estourar sombras ou altas luzes.
- **Correção de Lente (*Lens Correction*):** Compensação de distorção esférica (barril/almofada) e aberração cromática.
- **Remover Fundo (*Remove Background*):** Isolamento automático de sujeitos principais.

### 6.3 Filtro Completo Camera RAW
Painel integrado dedicado ao fluxo de revelação fotográfica:
- **Painel Básico / Luz:** Exposição, Contraste, Realces, Sombras, Brancos, Pretos.
- **Cor:** Temperatura de Cor (Kelvin), Matiz (*Tint*), Vibração (*Vibrance*) e Saturação.
- **Curvas Tone:** Curva paramétrica (Realces, Luzes, Escuros, Sombras) e Curva de pontos.
- **Mixer de Cores (HSL):** Ajuste individual de Matiz, Saturação e Luminância para cada uma das 8 faixas de cor (Vermelho, Laranja, Amarelo, Verde, Aqua, Azul, Roxo, Magenta).
- **Color Grading:** Rodas de cor separadas para Sombras, Meios-tons e Realces com controle de equilíbrio e mesclagem.
- **Detalhes:** Nitidez (*Sharpening*), Raio, Detalhe, Máscara e Redução de Ruído Luminante e Cromático.
- **Óptica:** Remoção de aberração cromática e vinheta de lente.
- **Geometria:** Correção de perspectiva vertical e horizontal, rotação, proporção, escala e deslocamento X/Y.

---

## 7. Canvas, Visualização e Geometria

- **Múltiplos Documentos em Abas:** Alternância fluida entre diferentes projetos abertos simultaneamente.
- **Navegação no Viewport:** Pan (Espaço+Arrastar ou ferramenta Mão), Zoom contínuo (Ctrl+Scroll, Z ou atalhos), ajuste à tela (Ctrl+0) e 100% de zoom (Ctrl+1).
- **Qualidade de Renderização:**
  - *Downsampling* de alta qualidade (bicúbico / lanczos) ao afastar o zoom para evitar moiré.
  - Grade de pixels (*Pixel Grid*) automática exibida em níveis altos de aproximação (>500%).
- **Réguas e Guias:**
  - Réguas horizontais e verticais em pixels (Ctrl+R).
  - Guias arrastáveis a partir das réguas com armazenamento no manifesto do projeto.
  - Grade de layout (*Layout Grid*) com espaçamento e subdivisões configuráveis.
- **Ferramenta de Corte (*Crop Tool*):**
  - Proporções pré-definidas (1:1, 16:9, 4:3, 9:16, 2:3, livre).
  - Corte simétrico (com tecla Alt).
  - Se houver seleção ativa, o corte inicia automaticamente alinhado à demarcação.
- **Redimensionamento:**
  - Tamanho da Tela de Pintura (*Canvas Size*): Redimensionamento do espaço de trabalho com ancoragem em 9 pontos.
  - Tamanho da Imagem (*Image Size*): Reamostragem de todo o documento com escolha de interpolação.
  - Aparar (*Trim*): Eliminação automática de bordas transparentes ou de cor sólida uniforme.

---

## 8. Interface do Usuário e Ergonomia (Linux / GTK4)

- **Design GNOME / Libadwaita:**
  - Janela moderna com `AdwHeaderBar` adaptativa, abas estilizadas e tema claro/escuro integrado com as preferências do sistema operacional.
- **Painéis Laterais:**
  - Painel de Ferramentas vertical à esquerda com ícones vetorizados e tooltips de atalho.
  - Cabeçalho de opções de ferramenta contextual no topo (ajuste de pincel, tolerância, etc.).
  - Painel de Camadas à direita com controle de opacidade, blend mode, visibilidade e miniaturas de camada/máscara.
  - Painéis recolhíveis para Histórico, Canais, Ajustes e Inspetor de Transformação.
- **Controle de Valores Numéricos (*Numeric Scrubbing*):**
  - Clique e arraste horizontal no rótulo de campos numéricos para aumentar ou diminuir o valor suavemente.
- **Atalhos de Teclado Padrão da Indústria:**
  - Totalmente mapeados para o padrão universal (Ctrl+Z, Ctrl+Shift+Z, B, E, V, M, L, W, T, G, C, I, H, Z, etc.).
- **Desempenho Responsivo:**
  - Operações pesadas (filtros, salvamento, exportação) executadas em threads assíncronas em segundo plano sem congelar a interface do usuário.
