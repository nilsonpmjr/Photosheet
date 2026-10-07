//! Janela Principal da Aplicação (MainWindow).
//! Traduzido de Compositor/CompositorApp.swift e WorkspaceView.swift.

use crate::core::adjustment::LayerAdjustment;
use crate::core::document::Document;
use crate::core::session::{EditorSession, NavigationTool};
use crate::ui::adjustments_panel::AdjustmentsPanel;
use crate::ui::canvas_widget::CanvasWidget;
use crate::ui::dialogs::{FilterDialogs, LevelsDialog, NewDocumentDialog, NewDocumentParams};
use crate::ui::header_bar::PhotosheetHeaderBar;
use crate::ui::layers_panel::LayersPanel;
use crate::ui::tool_bar::ToolBar;
use crate::ui::tool_options::ToolOptionsBar;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, DrawingArea, Label, Notebook, Orientation, Paned, Separator};
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PhotosheetWindow {
    pub window: libadwaita::ApplicationWindow,
    pub session: Rc<RefCell<EditorSession>>,
    pub header_bar: Rc<PhotosheetHeaderBar>,
    pub tool_bar: Rc<ToolBar>,
    pub tool_options: Rc<ToolOptionsBar>,
    pub layers_panel: Rc<LayersPanel>,
    pub adjustments_panel: Rc<AdjustmentsPanel>,
    pub tab_view: libadwaita::TabView,
    pub tab_bar: libadwaita::TabBar,
}

impl PhotosheetWindow {
    pub fn new(app: &libadwaita::Application) -> Self {
        let window = libadwaita::ApplicationWindow::builder()
            .application(app)
            .title("Photosheet")
            .default_width(1400)
            .default_height(900)
            .build();

        let session = Rc::new(RefCell::new(EditorSession::new()));
        let header_bar = Rc::new(PhotosheetHeaderBar::new());
        let tool_bar = Rc::new(ToolBar::new());
        let tool_options = Rc::new(ToolOptionsBar::new());
        let layers_panel = Rc::new(LayersPanel::new());
        let adjustments_panel = Rc::new(AdjustmentsPanel::new());

        let tab_view = libadwaita::TabView::new();
        let tab_bar = libadwaita::TabBar::new();
        tab_bar.set_view(Some(&tab_view));

        // Layout Principal: Vertical (HeaderBar, ToolOptions, TabBar, ContentArea)
        let root_box = GtkBox::new(Orientation::Vertical, 0);
        root_box.append(&header_bar.header_bar);
        root_box.append(&tool_options.container);
        root_box.append(&tab_bar);

        // Área Central Horizontal: (ToolBar à esquerda, TabView no meio, Painéis à direita)
        let center_box = GtkBox::new(Orientation::Horizontal, 0);
        center_box.set_vexpand(true);
        center_box.set_hexpand(true);

        center_box.append(&tool_bar.container);
        center_box.append(&Separator::new(Orientation::Vertical));

        // Paned central para dividir Canvas e Painel Lateral Direito
        let main_paned = Paned::new(Orientation::Horizontal);
        main_paned.set_start_child(Some(&tab_view));
        main_paned.set_resize_start_child(true);
        main_paned.set_shrink_start_child(false);

        // Notebook ou Stack para abas laterais (Camadas / Ajustes)
        let side_notebook = Notebook::new();
        side_notebook.append_page(&layers_panel.container, Some(&Label::new(Some("Camadas"))));
        side_notebook.append_page(
            &adjustments_panel.container,
            Some(&Label::new(Some("Ajustes"))),
        );

        main_paned.set_end_child(Some(&side_notebook));
        main_paned.set_resize_end_child(false);
        main_paned.set_shrink_end_child(false);
        main_paned.set_position(1100);

        center_box.append(&main_paned);
        root_box.append(&center_box);

        window.set_content(Some(&root_box));

        let photosheet_win = Self {
            window,
            session,
            header_bar,
            tool_bar,
            tool_options,
            layers_panel,
            adjustments_panel,
            tab_view,
            tab_bar,
        };

        photosheet_win.wire_signals();
        photosheet_win
    }

    pub fn present(&self) {
        self.window.present();
    }

    /// Cria uma nova aba de documento e inicializa a sessão de edição
    pub fn create_new_document(&self, params: NewDocumentParams) {
        let mut doc = Document::new(params.width, params.height, params.resolution);

        // Criar camada de fundo padrão
        let mut bg_layer = crate::core::layer::Layer::new_pixel(
            "Plano de Fundo".into(),
            params.width,
            params.height,
            crate::core::transform::LayerTransform::new(
                crate::core::transform::Point { x: 0.0, y: 0.0 },
                crate::core::transform::Size {
                    width: params.width as f64,
                    height: params.height as f64,
                },
            ),
        );

        if let Some(bg_color) = params.background_color {
            if let crate::core::layer::LayerKind::Pixel { ref mut pixels, .. } = bg_layer.kind {
                if let Some(buf) = pixels {
                    for chunk in buf.chunks_exact_mut(4) {
                        chunk.copy_from_slice(&bg_color);
                    }
                }
            }
        }

        doc.add_layer(bg_layer);

        // Configurar a sessão
        let zoom = {
            let mut session = self.session.borrow_mut();
            session.set_document(doc);
            session.zoom
        };

        // Criar widget do canvas interativo com suporte a pan, zoom e pincel
        let canvas_widget = CanvasWidget::new(Rc::clone(&self.session));
        let lp = Rc::clone(&self.layers_panel);
        let sess_ref = Rc::clone(&self.session);
        canvas_widget.connect_document_modified(move || {
            if let Some(doc) = &sess_ref.borrow().document {
                lp.update_layers(&doc.layers, doc.active_layer_id);
            }
        });

        let title = format!("Sem Título ({}×{})", params.width, params.height);
        let page = self.tab_view.append(&canvas_widget.container);
        page.set_title(&title);

        self.header_bar.set_title(&title);
        self.header_bar.set_zoom(zoom);

        // Atualizar lista de camadas na UI
        if let Some(doc_ref) = &self.session.borrow().document {
            self.layers_panel
                .update_layers(&doc_ref.layers, doc_ref.active_layer_id);
        }
    }

    fn wire_signals(&self) {
        // 1. Mudança de ferramenta via ToolBar
        {
            let session = Rc::clone(&self.session);
            let options = Rc::clone(&self.tool_options);
            self.tool_bar.connect_tool_selected(move |tool| {
                session.borrow_mut().active_tool = tool;
                options.set_active_tool(tool);
            });
        }

        // 2. Novo Documento
        {
            let win = self.window.clone();
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let header_bar = Rc::clone(&self.header_bar);
            let tab_view = self.tab_view.clone();

            self.header_bar.connect_new(move || {
                let win_clone = win.clone();
                let session = Rc::clone(&session);
                let layers_panel = Rc::clone(&layers_panel);
                let header_bar = Rc::clone(&header_bar);
                let tab_view = tab_view.clone();

                NewDocumentDialog::show(&win_clone, move |params| {
                    let mut doc = Document::new(params.width, params.height, params.resolution);
                    let bg_layer = crate::core::layer::Layer::new_pixel(
                        "Plano de Fundo".into(),
                        params.width,
                        params.height,
                        crate::core::transform::LayerTransform::new(
                            crate::core::transform::Point { x: 0.0, y: 0.0 },
                            crate::core::transform::Size {
                                width: params.width as f64,
                                height: params.height as f64,
                            },
                        ),
                    );
                    doc.add_layer(bg_layer);

                    let mut s = session.borrow_mut();
                    s.set_document(doc);

                    let canvas_widget = CanvasWidget::new(Rc::clone(&session));
                    let lp = Rc::clone(&layers_panel);
                    let sess_ref = Rc::clone(&session);
                    canvas_widget.connect_document_modified(move || {
                        if let Some(doc) = &sess_ref.borrow().document {
                            lp.update_layers(&doc.layers, doc.active_layer_id);
                        }
                    });

                    let title = format!("Sem Título ({}×{})", params.width, params.height);
                    let page = tab_view.append(&canvas_widget.container);
                    page.set_title(&title);

                    header_bar.set_title(&title);
                    header_bar.set_zoom(s.zoom);

                    if let Some(doc_ref) = &s.document {
                        layers_panel.update_layers(&doc_ref.layers, doc_ref.active_layer_id);
                    }
                });
            });
        }

        // 3. Desfazer e Refazer
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            self.header_bar.connect_undo(move || {
                let mut s = session.borrow_mut();
                if s.undo() {
                    if let Some(doc) = &s.document {
                        layers_panel.update_layers(&doc.layers, doc.active_layer_id);
                    }
                }
            });
        }
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            self.header_bar.connect_redo(move || {
                let mut s = session.borrow_mut();
                if s.redo() {
                    if let Some(doc) = &s.document {
                        layers_panel.update_layers(&doc.layers, doc.active_layer_id);
                    }
                }
            });
        }

        // Exportar Imagem
        {
            let win = self.window.clone();
            let session = Rc::clone(&self.session);
            self.header_bar.connect_export(move || {
                let s = session.borrow();
                if let Some(doc) = &s.document {
                    let sess_ref = Rc::clone(&session);
                    crate::ui::dialogs::ExportDialog::show(&win, doc, move |params| {
                        let s_mut = sess_ref.borrow();
                        if let Some(d) = &s_mut.document {
                            if let Ok(composite) =
                                crate::render::cpu_compositor::CpuCompositor::render(d)
                            {
                                let _ = crate::ui::dialogs::ExportDialog::export_composite(
                                    &composite.pixels,
                                    composite.width as u32,
                                    composite.height as u32,
                                    &params,
                                );
                            }
                        }
                    });
                }
            });
        }

        // 4. Ações de Camadas do LayersPanel
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            layers_panel.connect_add_layer(move || {
                let (layers, active_id) = {
                    let mut s = session.borrow_mut();
                    s.add_empty_layer(None);
                    match &s.document {
                        Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                        None => (Vec::new(), None),
                    }
                };
                lp.update_layers(&layers, active_id);
            });
        }
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            layers_panel.connect_move_layer_up(move || {
                let (layers, active_id) = {
                    let mut s = session.borrow_mut();
                    s.move_active_layer_up();
                    match &s.document {
                        Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                        None => (Vec::new(), None),
                    }
                };
                lp.update_layers(&layers, active_id);
            });
        }
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            layers_panel.connect_move_layer_down(move || {
                let (layers, active_id) = {
                    let mut s = session.borrow_mut();
                    s.move_active_layer_down();
                    match &s.document {
                        Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                        None => (Vec::new(), None),
                    }
                };
                lp.update_layers(&layers, active_id);
            });
        }
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            layers_panel.connect_select_layer(move |id| {
                let (changed, layers, active_id) = {
                    let mut s = session.borrow_mut();
                    let changed = s.select_layer(id);
                    let (layers, active_id) = match &s.document {
                        Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                        None => (Vec::new(), None),
                    };
                    (changed, layers, active_id)
                };
                if changed {
                    lp.update_layers(&layers, active_id);
                }
            });
        }
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            layers_panel.connect_toggle_lock(move |id| {
                let (layers, active_id) = {
                    let mut s = session.borrow_mut();
                    if let Some(doc) = s.document.as_mut() {
                        if let Some(layer) = doc.find_layer_mut(id) {
                            layer.is_locked = !layer.is_locked;
                            s.is_dirty = true;
                        }
                    }
                    match &s.document {
                        Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                        None => (Vec::new(), None),
                    }
                };
                lp.update_layers(&layers, active_id);
            });
        }
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            layers_panel.connect_add_group(move || {
                let (layers, active_id) = {
                    let mut s = session.borrow_mut();
                    s.add_group(None);
                    match &s.document {
                        Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                        None => (Vec::new(), None),
                    }
                };
                lp.update_layers(&layers, active_id);
            });
        }
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            layers_panel.connect_delete_layer(move || {
                let (layers, active_id) = {
                    let mut s = session.borrow_mut();
                    s.delete_active_layer();
                    match &s.document {
                        Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                        None => (Vec::new(), None),
                    }
                };
                lp.update_layers(&layers, active_id);
            });
        }
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            layers_panel.connect_duplicate_layer(move || {
                let (layers, active_id) = {
                    let mut s = session.borrow_mut();
                    s.duplicate_active_layer();
                    match &s.document {
                        Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                        None => (Vec::new(), None),
                    }
                };
                lp.update_layers(&layers, active_id);
            });
        }
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            layers_panel.connect_merge_down(move || {
                let (layers, active_id) = {
                    let mut s = session.borrow_mut();
                    s.merge_active_down();
                    match &s.document {
                        Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                        None => (Vec::new(), None),
                    }
                };
                lp.update_layers(&layers, active_id);
            });
        }
        {
            let session = Rc::clone(&self.session);
            self.layers_panel.connect_opacity_change(move |op| {
                let mut s = session.borrow_mut();
                s.set_active_layer_opacity(op);
            });
        }
        {
            let session = Rc::clone(&self.session);
            self.layers_panel.connect_blend_mode_change(move |mode| {
                let mut s = session.borrow_mut();
                s.set_active_layer_blend_mode(mode);
            });
        }
        {
            let session = Rc::clone(&self.session);
            self.tool_options.connect_shape_settings_change(move |settings| {
                session.borrow_mut().shape_settings = settings;
            });
        }
        {
            let session = Rc::clone(&self.session);
            self.tool_options.connect_text_settings_change(move |settings| {
                session.borrow_mut().text_settings = settings;
            });
        }
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            layers_panel.connect_toggle_visibility(move |id| {
                let (layers, active_id) = {
                    let mut s = session.borrow_mut();
                    if let Some(doc) = s.document.as_mut() {
                        if let Some(layer) = doc.find_layer_mut(id) {
                            layer.is_visible = !layer.is_visible;
                        }
                    }
                    match &s.document {
                        Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                        None => (Vec::new(), None),
                    }
                };
                lp.update_layers(&layers, active_id);
            });
        }

        // 5. Ajustes e Filtros
        {
            let session = Rc::clone(&self.session);
            let layers_panel = Rc::clone(&self.layers_panel);
            let lp = Rc::clone(&layers_panel);
            self.adjustments_panel
                .connect_adjustment_selected(move |adj| {
                    let (layers, active_id) = {
                        let mut s = session.borrow_mut();
                        s.add_adjustment_layer(adj, None);
                        match &s.document {
                            Some(doc) => (doc.layers.clone(), doc.active_layer_id),
                            None => (Vec::new(), None),
                        }
                    };
                    lp.update_layers(&layers, active_id);
                });
        }
    }
}
