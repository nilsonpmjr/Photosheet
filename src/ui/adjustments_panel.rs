//! Painel de Ajustes de Imagem e Cores (AdjustmentsPanel).
//! Traduzido de Compositor/UI/AdjustmentsPanel.swift.

use crate::core::adjustment::LayerAdjustment;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, ScrolledWindow, Separator};
use std::cell::RefCell;
use std::rc::Rc;

pub struct AdjustmentsPanel {
    pub container: GtkBox,
    on_adjustment_selected: Rc<RefCell<Option<Box<dyn Fn(LayerAdjustment)>>>>,
}

impl AdjustmentsPanel {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 6);
        container.set_margin_start(8);
        container.set_margin_end(8);
        container.set_margin_top(8);
        container.set_margin_bottom(8);
        container.set_width_request(260);

        let header = Label::builder()
            .label("Ajustes de Imagem")
            .halign(gtk4::Align::Start)
            .css_classes(["heading"])
            .build();
        container.append(&header);
        container.append(&Separator::new(Orientation::Horizontal));

        let adjustments = vec![
            (
                "Brilho / Contraste",
                "display-brightness-symbolic",
                LayerAdjustment::exposure(0.0, 0.0, 1.0),
            ),
            (
                "Níveis",
                "view-list-symbolic",
                LayerAdjustment::levels(0.0, 1.0, 255.0, 0.0, 255.0),
            ),
            (
                "Curvas",
                "network-cellular-signal-good-symbolic",
                LayerAdjustment::curves(),
            ),
            (
                "Exposição",
                "camera-flash-symbolic",
                LayerAdjustment::exposure(0.0, 0.0, 1.0),
            ),
            (
                "Matiz / Saturação",
                "color-gradient-symbolic",
                LayerAdjustment::hue_saturation(0.0, 0.0, 0.0),
            ),
            (
                "Equilíbrio de Cores",
                "weather-clear-symbolic",
                LayerAdjustment::new(crate::core::adjustment::AdjustmentKind::ColorBalance),
            ),
            (
                "Preto e Branco",
                "media-record-symbolic",
                LayerAdjustment::black_and_white(),
            ),
            (
                "Inverter",
                "view-refresh-symbolic",
                LayerAdjustment::invert(),
            ),
            (
                "Desfoque Gaussiano",
                "blur-symbolic",
                LayerAdjustment::gaussian_blur(5.0),
            ),
            (
                "Adicionar Ruído",
                "network-cellular-signal-none-symbolic",
                LayerAdjustment::add_noise(20.0, true, true),
            ),
            (
                "Mapa de Gradiente",
                "color-select-symbolic",
                LayerAdjustment::new(crate::core::adjustment::AdjustmentKind::GradientMap),
            ),
        ];

        let on_adjustment_selected = Rc::new(RefCell::new(None::<Box<dyn Fn(LayerAdjustment)>>));

        let list_box = GtkBox::new(Orientation::Vertical, 4);
        for (name, icon, adj) in adjustments {
            let btn = Button::builder()
                .label(name)
                .icon_name(icon)
                .halign(gtk4::Align::Fill)
                .build();
            btn.add_css_class("flat");

            let adj_clone = adj.clone();
            let cb = Rc::clone(&on_adjustment_selected);
            btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f(adj_clone.clone());
                }
            });

            list_box.append(&btn);
        }

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .child(&list_box)
            .build();

        container.append(&scrolled);

        Self {
            container,
            on_adjustment_selected,
        }
    }

    pub fn connect_adjustment_selected<F: Fn(LayerAdjustment) + 'static>(&self, f: F) {
        *self.on_adjustment_selected.borrow_mut() = Some(Box::new(f));
    }
}
