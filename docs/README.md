# Photoslop 🎨🐧

> Editor de imagens e composição não destrutivo, de alta performance e código aberto, nativo para **Linux**.
> Tradução e porte arquitetural do **[Compositor (macOS)](https://github.com/robbietilton/Compositor)** para **Rust**, **GTK4 / Libadwaita** e aceleração gráfica por hardware via **Vulkan** (AMD & NVIDIA).

---

## 📚 Documentação do Projeto

- **[Mapa Completo de Funcionalidades](FEATURE_MAP.md):** Levantamento exaustivo de todas as ferramentas, camadas, filtros e formatos do aplicativo original a serem portados.
- **[Especificação Técnica de Arquitetura (`/to-spec`)](SPEC.md):** Arquitetura de módulos, substituição de tecnologias (Metal/SwiftUI -> Vulkan/GTK4), modelo de concorrência e FFI com os kernels C.
- **[Plano de Tarefas e Decomposição em Tickets (`/to-tickets`)](TICKETS.md):** Decomposição em 22 tickets práticos e acionáveis, organizados em 6 fases de implementação.

---

## 🛠️ Stack Tecnológica

- **Linguagem do Core:** Rust (Edição 2021/2024)
- **Interface Gráfica:** GTK4 (4.22+) e Libadwaita (1.9+) via `gtk4-rs` / `libadwaita-rs`
- **Renderização e GPU:** Vulkan nativo via `wgpu-rs` (otimizado para GPUs AMD Mesa RADV e NVIDIA)
- **Processamento de Pixels na CPU:** Kernels em C99 puro compilados nativamente via `cc-rs`
- **Compatibilidade de Arquivos:** Suporte total a projetos `.comp` (v1–v11) e PSD/PSB de 8 bits
