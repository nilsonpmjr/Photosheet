//! Barra de Ferramentas Vertical (ToolBar).
//! Traduzido de Compositor/UI/NavigationToolHeader.swift e EditorSession.swift.

use crate::core::session::NavigationTool;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Orientation, Separator, ToggleButton};
use std::cell::RefCell;
use std::rc::Rc;

#[allow(dead_code)]
pub struct ToolBar {
    pub container: GtkBox,
    tool_buttons: Vec<(NavigationTool, ToggleButton)>,
    on_tool_selected: Rc<RefCell<Option<Box<dyn Fn(NavigationTool)>>>>,
    fg_swatch: Button,
    bg_swatch: Button,
}

impl ToolBar {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 4);
        container.set_margin_start(6);
        container.set_margin_end(6);
        container.set_margin_top(6);
        container.set_margin_bottom(6);
        container.add_css_class("toolbar");

        let tools = [
            NavigationTool::Move,
            NavigationTool::Marquee,
            NavigationTool::Lasso,
            NavigationTool::Wand,
            NavigationTool::Crop,
            NavigationTool::Brush,
            NavigationTool::Eraser,
            NavigationTool::SpotHealing,
            NavigationTool::CloneStamp,
            NavigationTool::Blur,
            NavigationTool::Gradient,
            NavigationTool::Shape,
            NavigationTool::Type,
            NavigationTool::Eyedropper,
            NavigationTool::Hand,
            NavigationTool::Zoom,
        ];

        let mut tool_buttons = Vec::new();
        let on_tool_selected = Rc::new(RefCell::new(None::<Box<dyn Fn(NavigationTool)>>));

        for tool in tools {
            let btn = ToggleButton::builder()
                .icon_name(tool.icon_name())
                .tooltip_text(tool.name())
                .build();
            btn.add_css_class("flat");

            if tool == NavigationTool::Move {
                btn.set_active(true);
            }

            let tool_copy = tool;
            let cb = Rc::clone(&on_tool_selected);
            btn.connect_toggled(move |b| {
                if b.is_active() {
                    if let Some(ref handler) = *cb.borrow() {
                        handler(tool_copy);
                    }
                }
            });

            container.append(&btn);
            tool_buttons.push((tool, btn));
        }

        container.append(&Separator::new(Orientation::Horizontal));

        // Área de Cores Foreground / Background
        let color_box = GtkBox::new(Orientation::Vertical, 2);
        color_box.set_halign(gtk4::Align::Center);

        let fg_swatch = Button::builder()
            .width_request(24)
            .height_request(24)
            .tooltip_text("Cor de Primeiro Plano (Pressione X para alternar)")
            .build();
        fg_swatch.add_css_class("flat");

        let bg_swatch = Button::builder()
            .width_request(24)
            .height_request(24)
            .tooltip_text("Cor do Fundo")
            .build();
        bg_swatch.add_css_class("flat");

        let swap_btn = Button::builder()
            .icon_name("object-flip-horizontal-symbolic")
            .tooltip_text("Alternar Cores (X)")
            .build();
        swap_btn.add_css_class("flat");

        let reset_btn = Button::builder()
            .icon_name("edit-clear-all-symbolic")
            .tooltip_text("Restaurar Cores Padrão (D)")
            .build();
        reset_btn.add_css_class("flat");

        color_box.append(&fg_swatch);
        color_box.append(&bg_swatch);
        color_box.append(&swap_btn);
        color_box.append(&reset_btn);

        container.append(&color_box);

        Self {
            container,
            tool_buttons,
            on_tool_selected,
            fg_swatch,
            bg_swatch,
        }
    }

    pub fn set_active_tool(&self, tool: NavigationTool) {
        for (t, btn) in &self.tool_buttons {
            btn.set_active(*t == tool);
        }
    }

    pub fn connect_tool_selected<F: Fn(NavigationTool) + 'static>(&self, f: F) {
        *self.on_tool_selected.borrow_mut() = Some(Box::new(f));
    }
}
