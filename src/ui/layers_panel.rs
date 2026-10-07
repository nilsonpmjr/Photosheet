use crate::core::blend::LayerBlendMode;
use crate::core::layer::Layer;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, DropDown, Image, Label, ListBox, ListBoxRow, Orientation,
    Scale, ScrolledWindow, Separator, StringList,
};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

#[allow(dead_code)]
pub struct LayersPanel {
    pub container: GtkBox,
    blend_dropdown: DropDown,
    opacity_scale: Scale,
    layers_list: ListBox,
    on_add_layer: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_add_group: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_add_adjustment: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_delete_layer: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_duplicate_layer: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_merge_down: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_opacity_change: Rc<RefCell<Option<Box<dyn Fn(f64)>>>>,
    on_blend_mode_change: Rc<RefCell<Option<Box<dyn Fn(LayerBlendMode)>>>>,
    on_select_layer: Rc<RefCell<Option<Box<dyn Fn(Uuid)>>>>,
    on_toggle_visibility: Rc<RefCell<Option<Box<dyn Fn(Uuid)>>>>,
    on_toggle_lock: Rc<RefCell<Option<Box<dyn Fn(Uuid)>>>>,
}

impl LayersPanel {
    pub const BLEND_MODES: &'static [(&'static str, LayerBlendMode)] = &[
        ("Normal", LayerBlendMode::Normal),
        ("Darken", LayerBlendMode::Darken),
        ("Multiply", LayerBlendMode::Multiply),
        ("Color Burn", LayerBlendMode::ColorBurn),
        ("Linear Burn", LayerBlendMode::LinearBurn),
        ("Lighten", LayerBlendMode::Lighten),
        ("Screen", LayerBlendMode::Screen),
        ("Color Dodge", LayerBlendMode::ColorDodge),
        ("Linear Dodge (Add)", LayerBlendMode::LinearDodge),
        ("Overlay", LayerBlendMode::Overlay),
        ("Soft Light", LayerBlendMode::SoftLight),
        ("Hard Light", LayerBlendMode::HardLight),
        ("Vivid Light", LayerBlendMode::VividLight),
        ("Linear Light", LayerBlendMode::LinearLight),
        ("Pin Light", LayerBlendMode::PinLight),
        ("Hard Mix", LayerBlendMode::HardMix),
        ("Difference", LayerBlendMode::Difference),
        ("Exclusion", LayerBlendMode::Exclusion),
        ("Subtract", LayerBlendMode::Subtract),
        ("Divide", LayerBlendMode::Divide),
        ("Hue", LayerBlendMode::Hue),
        ("Saturation", LayerBlendMode::Saturation),
        ("Color", LayerBlendMode::Color),
        ("Luminosity", LayerBlendMode::Luminosity),
    ];

    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 6);
        container.set_margin_start(8);
        container.set_margin_end(8);
        container.set_margin_top(8);
        container.set_margin_bottom(8);
        container.set_width_request(260);

        // Cabeçalho do Painel
        let header = Label::builder()
            .label("Camadas")
            .halign(gtk4::Align::Start)
            .css_classes(["heading"])
            .build();
        container.append(&header);

        // Controles de Modo de Mesclagem e Opacidade
        let top_controls = GtkBox::new(Orientation::Vertical, 6);

        let blend_box = GtkBox::new(Orientation::Horizontal, 6);
        blend_box.append(&Label::new(Some("Mesclagem:")));

        let blend_names: Vec<&str> = Self::BLEND_MODES.iter().map(|(n, _)| *n).collect();
        let blend_strings = StringList::new(&blend_names);
        let blend_dropdown = DropDown::new(Some(blend_strings), gtk4::Expression::NONE);
        blend_dropdown.set_hexpand(true);
        blend_box.append(&blend_dropdown);
        top_controls.append(&blend_box);

        let opacity_box = GtkBox::new(Orientation::Horizontal, 6);
        opacity_box.append(&Label::new(Some("Opacidade:")));
        let opacity_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        opacity_scale.set_value(100.0);
        opacity_scale.set_hexpand(true);
        opacity_scale.set_draw_value(true);
        opacity_box.append(&opacity_scale);
        top_controls.append(&opacity_box);

        container.append(&top_controls);
        container.append(&Separator::new(Orientation::Horizontal));

        // Lista de Camadas com Rolagem
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let layers_list = ListBox::new();
        layers_list.add_css_class("boxed-list");
        scrolled.set_child(Some(&layers_list));
        container.append(&scrolled);

        container.append(&Separator::new(Orientation::Horizontal));

        // Barra de Ações na Base do Painel
        let bottom_actions = GtkBox::new(Orientation::Horizontal, 4);
        bottom_actions.set_halign(gtk4::Align::End);

        let add_layer_btn = Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Nova Camada")
            .build();
        add_layer_btn.add_css_class("flat");

        let add_group_btn = Button::builder()
            .icon_name("folder-new-symbolic")
            .tooltip_text("Novo Grupo")
            .build();
        add_group_btn.add_css_class("flat");

        let add_adj_btn = Button::builder()
            .icon_name("display-brightness-symbolic")
            .tooltip_text("Nova Camada de Ajuste")
            .build();
        add_adj_btn.add_css_class("flat");

        let dup_layer_btn = Button::builder()
            .icon_name("edit-copy-symbolic")
            .tooltip_text("Duplicar Camada")
            .build();
        dup_layer_btn.add_css_class("flat");

        let delete_layer_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text("Excluir Camada")
            .build();
        delete_layer_btn.add_css_class("flat");

        let merge_down_btn = Button::builder()
            .icon_name("go-down-symbolic")
            .tooltip_text("Mesclar Camada Abaixo")
            .build();
        merge_down_btn.add_css_class("flat");

        bottom_actions.append(&add_layer_btn);
        bottom_actions.append(&add_group_btn);
        bottom_actions.append(&add_adj_btn);
        bottom_actions.append(&dup_layer_btn);
        bottom_actions.append(&merge_down_btn);
        bottom_actions.append(&delete_layer_btn);

        container.append(&bottom_actions);

        let on_add_layer = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_add_group = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_add_adjustment = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_delete_layer = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_duplicate_layer = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_merge_down = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_opacity_change = Rc::new(RefCell::new(None::<Box<dyn Fn(f64)>>));
        let on_blend_mode_change = Rc::new(RefCell::new(None::<Box<dyn Fn(LayerBlendMode)>>));
        let on_select_layer = Rc::new(RefCell::new(None::<Box<dyn Fn(Uuid)>>));
        let on_toggle_visibility = Rc::new(RefCell::new(None::<Box<dyn Fn(Uuid)>>));
        let on_toggle_lock = Rc::new(RefCell::new(None::<Box<dyn Fn(Uuid)>>));

        {
            let cb = Rc::clone(&on_select_layer);
            layers_list.connect_row_selected(move |_list, row| {
                let Some(row) = row else {
                    return;
                };
                let Ok(id) = Uuid::parse_str(&row.widget_name()) else {
                    return;
                };
                if let Some(ref f) = *cb.borrow() {
                    f(id);
                }
            });
        }

        // Callbacks de botões
        {
            let cb = Rc::clone(&on_add_layer);
            add_layer_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }
        {
            let cb = Rc::clone(&on_add_group);
            add_group_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }
        {
            let cb = Rc::clone(&on_add_adjustment);
            add_adj_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }
        {
            let cb = Rc::clone(&on_duplicate_layer);
            dup_layer_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }
        {
            let cb = Rc::clone(&on_delete_layer);
            delete_layer_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }
        {
            let cb = Rc::clone(&on_merge_down);
            merge_down_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }

        // Opacidade slider
        {
            let cb = Rc::clone(&on_opacity_change);
            opacity_scale.connect_value_changed(move |s| {
                if let Some(ref f) = *cb.borrow() {
                    f(s.value() / 100.0);
                }
            });
        }

        // Blend mode dropdown
        {
            let cb = Rc::clone(&on_blend_mode_change);
            blend_dropdown.connect_selected_notify(move |d| {
                let idx = d.selected() as usize;
                if idx < Self::BLEND_MODES.len() {
                    let mode = Self::BLEND_MODES[idx].1;
                    if let Some(ref f) = *cb.borrow() {
                        f(mode);
                    }
                }
            });
        }

        Self {
            container,
            blend_dropdown,
            opacity_scale,
            layers_list,
            on_add_layer,
            on_add_group,
            on_add_adjustment,
            on_delete_layer,
            on_duplicate_layer,
            on_merge_down,
            on_opacity_change,
            on_blend_mode_change,
            on_select_layer,
            on_toggle_visibility,
            on_toggle_lock,
        }
    }

    /// Atualiza a lista visual de camadas a partir das camadas do documento.
    pub fn update_layers(&self, layers: &[Layer], active_id: Option<Uuid>) {
        // Limpar itens anteriores
        while let Some(child) = self.layers_list.first_child() {
            self.layers_list.remove(&child);
        }

        // Inverter a ordem para exibição: a camada superior aparece no topo da lista UI
        for layer in layers.iter().rev() {
            let row = ListBoxRow::new();
            row.set_widget_name(&layer.id.to_string());
            let row_box = GtkBox::new(Orientation::Horizontal, 8);
            row_box.set_margin_start(8);
            row_box.set_margin_end(8);
            row_box.set_margin_top(4);
            row_box.set_margin_bottom(4);

            // Botão de Visibilidade (Olho)
            let vis_icon = if layer.is_visible {
                "view-reveal-symbolic"
            } else {
                "view-conceal-symbolic"
            };
            let vis_btn = Button::builder()
                .icon_name(vis_icon)
                .tooltip_text("Visibilidade da Camada")
                .build();
            vis_btn.add_css_class("flat");

            let id = layer.id;
            let cb_vis = Rc::clone(&self.on_toggle_visibility);
            vis_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb_vis.borrow() {
                    f(id);
                }
            });
            row_box.append(&vis_btn);

            let lock_icon = if layer.is_locked { "changes-prevent-symbolic" } else { "changes-allow-symbolic" };
            let lock_btn = Button::builder()
                .icon_name(lock_icon)
                .tooltip_text(if layer.is_locked { "Desbloquear Camada" } else { "Bloquear Camada" })
                .build();
            lock_btn.add_css_class("flat");
            let cb_lock = Rc::clone(&self.on_toggle_lock);
            lock_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb_lock.borrow() {
                    f(id);
                }
            });
            row_box.append(&lock_btn);

            // Ícone do tipo da camada
            let type_icon = if layer.is_group {
                "folder-symbolic"
            } else {
                "image-x-generic-symbolic"
            };
            let img = Image::from_icon_name(type_icon);
            row_box.append(&img);

            // Nome da Camada
            let label = Label::builder()
                .label(&layer.name)
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .build();
            row_box.append(&label);

            row.set_child(Some(&row_box));

            if Some(layer.id) == active_id {
                self.layers_list.select_row(Some(&row));
            }

            self.layers_list.append(&row);
        }
    }

    pub fn connect_add_layer<F: Fn() + 'static>(&self, f: F) {
        *self.on_add_layer.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_add_group<F: Fn() + 'static>(&self, f: F) {
        *self.on_add_group.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_add_adjustment<F: Fn() + 'static>(&self, f: F) {
        *self.on_add_adjustment.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_delete_layer<F: Fn() + 'static>(&self, f: F) {
        *self.on_delete_layer.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_duplicate_layer<F: Fn() + 'static>(&self, f: F) {
        *self.on_duplicate_layer.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_merge_down<F: Fn() + 'static>(&self, f: F) {
        *self.on_merge_down.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_opacity_change<F: Fn(f64) + 'static>(&self, f: F) {
        *self.on_opacity_change.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_blend_mode_change<F: Fn(LayerBlendMode) + 'static>(&self, f: F) {
        *self.on_blend_mode_change.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_select_layer<F: Fn(Uuid) + 'static>(&self, f: F) {
        *self.on_select_layer.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_toggle_visibility<F: Fn(Uuid) + 'static>(&self, f: F) {
        *self.on_toggle_visibility.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_toggle_lock<F: Fn(Uuid) + 'static>(&self, f: F) {
        *self.on_toggle_lock.borrow_mut() = Some(Box::new(f));
    }
}
