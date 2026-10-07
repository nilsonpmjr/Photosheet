//! Barra de Opções Contextuais da Ferramenta Ativa (ToolOptions).
//! Traduzido de Compositor/UI/NavigationToolHeader.swift.

use crate::core::session::{
    BrushSettings, NavigationTool, ShapeKind, ShapeToolSettings, TextToolSettings, WandSettings,
};
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Box as GtkBox, CheckButton, DropDown, Label, Orientation, Scale, Separator,
    SpinButton, StringList,
};
use std::cell::RefCell;
use std::rc::Rc;

#[allow(dead_code)]
pub struct ToolOptionsBar {
    pub container: GtkBox,
    options_stack: gtk4::Stack,
    brush_box: GtkBox,
    wand_box: GtkBox,
    selection_box: GtkBox,
    shape_box: GtkBox,
    text_box: GtkBox,
    crop_box: GtkBox,
    move_box: GtkBox,
    shape_settings: Rc<RefCell<ShapeToolSettings>>,
    on_shape_settings_change: Rc<RefCell<Option<Box<dyn Fn(ShapeToolSettings)>>>>,
    text_settings: Rc<RefCell<TextToolSettings>>,
    on_text_settings_change: Rc<RefCell<Option<Box<dyn Fn(TextToolSettings)>>>>,
}

impl ToolOptionsBar {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(4);
        container.set_margin_bottom(4);
        container.add_css_class("toolbar");

        let options_stack = gtk4::Stack::new();
        options_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

        // 1. Opções do Pincel / Borracha
        let brush_box = GtkBox::new(Orientation::Horizontal, 8);
        brush_box.append(&Label::new(Some("Tamanho:")));
        let size_adj = Adjustment::new(30.0, 1.0, 1000.0, 1.0, 10.0, 0.0);
        let size_spin = SpinButton::new(Some(&size_adj), 1.0, 0);
        brush_box.append(&size_spin);

        brush_box.append(&Separator::new(Orientation::Vertical));
        brush_box.append(&Label::new(Some("Dureza:")));
        let hardness_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        hardness_scale.set_value(80.0);
        hardness_scale.set_width_request(100);
        brush_box.append(&hardness_scale);

        brush_box.append(&Separator::new(Orientation::Vertical));
        brush_box.append(&Label::new(Some("Opacidade:")));
        let opacity_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        opacity_scale.set_value(100.0);
        opacity_scale.set_width_request(100);
        brush_box.append(&opacity_scale);

        options_stack.add_named(&brush_box, Some("brush"));

        // 2. Opções da Varinha Mágica
        let wand_box = GtkBox::new(Orientation::Horizontal, 8);
        wand_box.append(&Label::new(Some("Tolerância:")));
        let tol_adj = Adjustment::new(32.0, 0.0, 255.0, 1.0, 8.0, 0.0);
        let tol_spin = SpinButton::new(Some(&tol_adj), 1.0, 0);
        wand_box.append(&tol_spin);

        let contiguous_check = CheckButton::with_label("Contíguo");
        contiguous_check.set_active(true);
        wand_box.append(&contiguous_check);

        let antialias_check = CheckButton::with_label("Suavização");
        antialias_check.set_active(true);
        wand_box.append(&antialias_check);

        options_stack.add_named(&wand_box, Some("wand"));

        // 3. Opções de Seleção (Marquee / Lasso)
        let selection_box = GtkBox::new(Orientation::Horizontal, 8);
        selection_box.append(&Label::new(Some("Difusão:")));
        let feather_adj = Adjustment::new(0.0, 0.0, 100.0, 1.0, 5.0, 0.0);
        let feather_spin = SpinButton::new(Some(&feather_adj), 1.0, 0);
        selection_box.append(&feather_spin);
        selection_box.append(&Label::new(Some("px")));

        options_stack.add_named(&selection_box, Some("selection"));

        // 4. Opções de Forma (Shape)
        let shape_box = GtkBox::new(Orientation::Horizontal, 8);
        let shape_settings = Rc::new(RefCell::new(ShapeToolSettings::default()));
        let on_shape_settings_change = Rc::new(RefCell::new(None::<Box<dyn Fn(ShapeToolSettings)>>));
        let shape_kinds = StringList::new(&["Retângulo", "Elipse", "Linha"]);
        let shape_dropdown = DropDown::new(Some(shape_kinds), gtk4::Expression::NONE);
        shape_box.append(&Label::new(Some("Forma:")));
        shape_box.append(&shape_dropdown);

        shape_box.append(&Label::new(Some("Raio do Canto:")));
        let corner_adj = Adjustment::new(0.0, 0.0, 500.0, 1.0, 5.0, 0.0);
        let corner_spin = SpinButton::new(Some(&corner_adj), 1.0, 0);
        shape_box.append(&corner_spin);

        shape_box.append(&Label::new(Some("Traçado:")));
        let stroke_adj = Adjustment::new(0.0, 0.0, 100.0, 1.0, 2.0, 0.0);
        let stroke_spin = SpinButton::new(Some(&stroke_adj), 1.0, 0);
        shape_box.append(&stroke_spin);

        {
            let settings = Rc::clone(&shape_settings);
            let callback = Rc::clone(&on_shape_settings_change);
            shape_dropdown.connect_selected_notify(move |dropdown| {
                settings.borrow_mut().kind = match dropdown.selected() {
                    1 => ShapeKind::Ellipse,
                    2 => ShapeKind::Line,
                    _ => ShapeKind::Rectangle,
                };
                if let Some(callback) = callback.borrow().as_ref() {
                    callback(settings.borrow().clone());
                }
            });
        }
        {
            let settings = Rc::clone(&shape_settings);
            let callback = Rc::clone(&on_shape_settings_change);
            corner_spin.connect_value_changed(move |spin| {
                settings.borrow_mut().corner_radius = spin.value();
                if let Some(callback) = callback.borrow().as_ref() {
                    callback(settings.borrow().clone());
                }
            });
        }
        {
            let settings = Rc::clone(&shape_settings);
            let callback = Rc::clone(&on_shape_settings_change);
            stroke_spin.connect_value_changed(move |spin| {
                settings.borrow_mut().stroke_width = spin.value();
                if let Some(callback) = callback.borrow().as_ref() {
                    callback(settings.borrow().clone());
                }
            });
        }

        options_stack.add_named(&shape_box, Some("shape"));

        // 5. Opções de Texto
        let text_box = GtkBox::new(Orientation::Horizontal, 8);
        let text_settings = Rc::new(RefCell::new(TextToolSettings::default()));
        let on_text_settings_change = Rc::new(RefCell::new(None::<Box<dyn Fn(TextToolSettings)>>));
        text_box.append(&Label::new(Some("Tamanho da Fonte:")));
        let font_adj = Adjustment::new(36.0, 6.0, 288.0, 1.0, 6.0, 0.0);
        let font_spin = SpinButton::new(Some(&font_adj), 1.0, 0);
        text_box.append(&font_spin);
        text_box.append(&Label::new(Some("pt")));

        {
            let settings = Rc::clone(&text_settings);
            let callback = Rc::clone(&on_text_settings_change);
            font_spin.connect_value_changed(move |spin| {
                settings.borrow_mut().font_size = spin.value();
                if let Some(callback) = callback.borrow().as_ref() {
                    callback(settings.borrow().clone());
                }
            });
        }

        options_stack.add_named(&text_box, Some("text"));

        // 6. Opções de Corte (Crop)
        let crop_box = GtkBox::new(Orientation::Horizontal, 8);
        crop_box.append(&Label::new(Some("Proporção:")));
        let crop_ratios = StringList::new(&["Livre", "1:1 (Quadrado)", "16:9", "4:3", "3:2"]);
        let crop_dropdown = DropDown::new(Some(crop_ratios), gtk4::Expression::NONE);
        crop_box.append(&crop_dropdown);

        let apply_crop_btn = gtk4::Button::with_label("Aplicar Corte");
        crop_box.append(&apply_crop_btn);

        options_stack.add_named(&crop_box, Some("crop"));

        // 7. Opções de Mover (Move)
        let move_box = GtkBox::new(Orientation::Horizontal, 8);
        let auto_select = CheckButton::with_label("Seleção Automática");
        let show_controls = CheckButton::with_label("Controles de Transformação");
        show_controls.set_active(true);
        move_box.append(&auto_select);
        move_box.append(&show_controls);

        options_stack.add_named(&move_box, Some("move"));

        container.append(&options_stack);

        Self {
            container,
            options_stack,
            brush_box,
            wand_box,
            selection_box,
            shape_box,
            text_box,
            crop_box,
            move_box,
            shape_settings,
            on_shape_settings_change,
            text_settings,
            on_text_settings_change,
        }
    }

    pub fn set_active_tool(&self, tool: NavigationTool) {
        let name = match tool {
            NavigationTool::Brush
            | NavigationTool::Eraser
            | NavigationTool::SpotHealing
            | NavigationTool::CloneStamp
            | NavigationTool::Blur => "brush",
            NavigationTool::Wand => "wand",
            NavigationTool::Marquee | NavigationTool::Lasso => "selection",
            NavigationTool::Shape => "shape",
            NavigationTool::Type => "text",
            NavigationTool::Crop => "crop",
            NavigationTool::Move => "move",
            _ => "move",
        };
        self.options_stack.set_visible_child_name(name);
    }

    pub fn connect_shape_settings_change<F: Fn(ShapeToolSettings) + 'static>(&self, f: F) {
        *self.on_shape_settings_change.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_text_settings_change<F: Fn(TextToolSettings) + 'static>(&self, f: F) {
        *self.on_text_settings_change.borrow_mut() = Some(Box::new(f));
    }
}
