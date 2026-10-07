//! Widget do Canvas Interativo (CanvasWidget).
//! Traduzido de Compositor/Rendering/EditorCanvas.swift, GPUCanvas.swift e CanvasRulers.swift.

use crate::core::session::{EditorSession, NavigationTool};
use crate::core::tools::{
    BrushEngine, CloneStampEngine, HealEngine, ShapeEngine, SpotHealingMode, TransformGizmo,
    TransformHandle,
};
use crate::core::transform::Point;
use gtk4::cairo::{Context, Format, ImageSurface};
use gtk4::gdk::{Key, ModifierType};
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, DrawingArea, EventControllerKey, EventControllerMotion, EventControllerScroll,
    GestureClick, GestureDrag, Orientation,
};
use std::cell::RefCell;
use std::rc::Rc;

#[allow(dead_code)]
pub struct CanvasWidget {
    pub container: GtkBox,
    pub drawing_area: DrawingArea,
    pub session: Rc<RefCell<EditorSession>>,
    drag_start: Rc<RefCell<Option<(f64, f64)>>>,
    last_mouse_pos: Rc<RefCell<(f64, f64)>>,
    active_handle: Rc<RefCell<Option<TransformHandle>>>,
    on_document_modified: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl CanvasWidget {
    pub fn new(session: Rc<RefCell<EditorSession>>) -> Self {
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.set_vexpand(true);
        container.set_hexpand(true);

        let drawing_area = DrawingArea::new();
        drawing_area.set_vexpand(true);
        drawing_area.set_hexpand(true);
        drawing_area.set_can_focus(true);

        let drag_start = Rc::new(RefCell::new(None));
        let last_mouse_pos = Rc::new(RefCell::new((0.0, 0.0)));
        let active_handle = Rc::new(RefCell::new(None));
        let on_document_modified = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));

        // 1. Função de Desenho do Canvas (Draw Callback)
        {
            let sess = Rc::clone(&session);
            let mouse_pos = Rc::clone(&last_mouse_pos);
            drawing_area.set_draw_func(move |_area, cr, width, height| {
                Self::draw_canvas(
                    &sess.borrow(),
                    cr,
                    width as f64,
                    height as f64,
                    *mouse_pos.borrow(),
                );
            });
        }

        // 2. Controlador de Arrastar (GestureDrag) para Pintura, Mover e Pan
        {
            let drag = GestureDrag::new();
            let sess = Rc::clone(&session);
            let area_clone = drawing_area.clone();
            let d_start = Rc::clone(&drag_start);
            let handle_ref = Rc::clone(&active_handle);
            let on_mod = Rc::clone(&on_document_modified);

            // Início do arrasto
            {
                let sess = Rc::clone(&sess);
                let d_start = Rc::clone(&d_start);
                let handle_ref = Rc::clone(&handle_ref);
                let area = area_clone.clone();

                drag.connect_drag_begin(move |_gesture, x, y| {
                    area.grab_focus();
                    *d_start.borrow_mut() = Some((x, y));

                    let mut s = sess.borrow_mut();
                    let (doc_x, doc_y) =
                        Self::screen_to_doc(&s, x, y, area.width() as f64, area.height() as f64);

                    if s.active_tool == NavigationTool::Move {
                        if let Some(doc) = &s.document {
                            if let Some(active_layer) = doc.active_layer() {
                                let hit = TransformGizmo::hit_test(
                                    &active_layer.transform,
                                    Point { x: doc_x, y: doc_y },
                                );
                                *handle_ref.borrow_mut() = hit;
                            }
                        }
                    } else if s.active_tool == NavigationTool::Brush
                        || s.active_tool == NavigationTool::Eraser
                    {
                        let is_eraser = s.active_tool == NavigationTool::Eraser;
                        let color = s.foreground_color;
                        let settings = s.brush_settings.clone();

                        if let Some(doc) = s.document.as_mut() {
                            let w = doc.width;
                            let h = doc.height;
                            if let Some(layer) = doc.active_layer_mut() {
                                if let Some(buf) = layer.raster_pixels_mut() {
                                    BrushEngine::render_dab(
                                        doc_x, doc_y, &settings, color, is_eraser, buf, w, h,
                                        None,
                                    );
                                }
                            }
                        }
                        area.queue_draw();
                    }
                });
            }

            // Atualização contínua do arrasto
            {
                let sess = Rc::clone(&sess);
                let d_start = Rc::clone(&d_start);
                let handle_ref = Rc::clone(&handle_ref);
                let area = area_clone.clone();

                drag.connect_drag_update(move |_gesture, offset_x, offset_y| {
                    let start = match *d_start.borrow() {
                        Some(p) => p,
                        None => return,
                    };

                    let curr_x = start.0 + offset_x;
                    let curr_y = start.1 + offset_y;

                    let mut s = sess.borrow_mut();
                    let (curr_doc_x, curr_doc_y) = Self::screen_to_doc(
                        &s,
                        curr_x,
                        curr_y,
                        area.width() as f64,
                        area.height() as f64,
                    );
                    let (prev_doc_x, prev_doc_y) = Self::screen_to_doc(
                        &s,
                        curr_x - offset_x,
                        curr_y - offset_y,
                        area.width() as f64,
                        area.height() as f64,
                    );

                    if s.active_tool == NavigationTool::Hand {
                        s.pan_x += offset_x / s.zoom;
                        s.pan_y += offset_y / s.zoom;
                        area.queue_draw();
                    } else if s.active_tool == NavigationTool::Brush
                        || s.active_tool == NavigationTool::Eraser
                    {
                        let is_eraser = s.active_tool == NavigationTool::Eraser;
                        let color = s.foreground_color;
                        let settings = s.brush_settings.clone();

                        if let Some(doc) = s.document.as_mut() {
                            let w = doc.width;
                            let h = doc.height;
                            if let Some(layer) = doc.active_layer_mut() {
                                if let Some(buf) = layer.raster_pixels_mut() {
                                    BrushEngine::stroke_line(
                                        prev_doc_x, prev_doc_y, curr_doc_x, curr_doc_y,
                                        &settings, color, is_eraser, buf, w, h, None,
                                    );
                                }
                            }
                        }
                        area.queue_draw();
                    } else if s.active_tool == NavigationTool::Move {
                        if let Some(handle) = *handle_ref.borrow() {
                            let d_doc_x = curr_doc_x - prev_doc_x;
                            let d_doc_y = curr_doc_y - prev_doc_y;
                            if let Some(doc) = s.document.as_mut() {
                                if let Some(layer) = doc.active_layer_mut() {
                                    TransformGizmo::apply_drag(
                                        &mut layer.transform,
                                        handle,
                                        d_doc_x,
                                        d_doc_y,
                                        false,
                                    );
                                }
                            }
                            area.queue_draw();
                        }
                    }
                });
            }

            // Final do arrasto
            {
                let sess = Rc::clone(&sess);
                let d_start = Rc::clone(&d_start);
                let handle_ref = Rc::clone(&handle_ref);
                let on_mod = Rc::clone(&on_mod);
                let area = area_clone.clone();

                drag.connect_drag_end(move |_gesture, offset_x, offset_y| {
                    if let Some(start) = *d_start.borrow() {
                        {
                            let mut s = sess.borrow_mut();
                            if s.active_tool == NavigationTool::Brush
                                || s.active_tool == NavigationTool::Eraser
                            {
                                s.push_history("Pincelada");
                            } else if s.active_tool == NavigationTool::Move {
                                s.push_history("Mover / Transformar");
                            } else if s.active_tool == NavigationTool::Shape {
                                let (start_x, start_y) = Self::screen_to_doc(
                                    &s, start.0, start.1, area.width() as f64, area.height() as f64,
                                );
                                let (end_x, end_y) = Self::screen_to_doc(
                                    &s, start.0 + offset_x, start.1 + offset_y,
                                    area.width() as f64, area.height() as f64,
                                );
                                s.add_shape(
                                    Point { x: start_x, y: start_y },
                                    Point { x: end_x, y: end_y },
                                );
                            }
                        }
                        area.queue_draw();
                        if let Some(ref cb) = *on_mod.borrow() {
                            cb();
                        }
                    }
                    *d_start.borrow_mut() = None;
                    *handle_ref.borrow_mut() = None;
                });
            }

            drawing_area.add_controller(drag);
        }

        // 3. Clique com a ferramenta Texto cria uma layer editável com o
        // fallback raster que será salvo no pacote `.comp`.
        {
            let click = GestureClick::new();
            let sess = Rc::clone(&session);
            let area = drawing_area.clone();
            let on_mod = Rc::clone(&on_document_modified);
            click.connect_pressed(move |_gesture, _count, x, y| {
                let mut s = sess.borrow_mut();
                if s.active_tool != NavigationTool::Type {
                    return;
                }
                let (doc_x, doc_y) = Self::screen_to_doc(
                    &s, x, y, area.width() as f64, area.height() as f64,
                );
                if s.add_text(Point { x: doc_x, y: doc_y }, "Text".to_string()).is_some() {
                    area.queue_draw();
                    if let Some(callback) = on_mod.borrow().as_ref() {
                        callback();
                    }
                }
            });
            drawing_area.add_controller(click);
        }

        // 4. Controlador de Rolagem (Zoom com Roda do Mouse)
        {
            let scroll = EventControllerScroll::new(gtk4::EventControllerScrollFlags::BOTH_AXES);
            let sess = Rc::clone(&session);
            let area_clone = drawing_area.clone();

            scroll.connect_scroll(move |_controller, _dx, dy| {
                let mut s = sess.borrow_mut();
                if dy < 0.0 {
                    s.zoom = (s.zoom * 1.15).min(64.0);
                } else if dy > 0.0 {
                    s.zoom = (s.zoom / 1.15).max(0.01);
                }
                area_clone.queue_draw();
                gtk4::glib::Propagation::Stop
            });

            drawing_area.add_controller(scroll);
        }

        // 4. Controlador de Movimento do Cursor (Mouse Motion para Contorno do Pincel)
        {
            let motion = EventControllerMotion::new();
            let mouse_pos = Rc::clone(&last_mouse_pos);
            let area_clone = drawing_area.clone();

            motion.connect_motion(move |_ctrl, x, y| {
                *mouse_pos.borrow_mut() = (x, y);
                area_clone.queue_draw();
            });

            drawing_area.add_controller(motion);
        }

        // 5. Controlador de Teclas (Atalhos no Canvas: B, E, V, Espaço, Zoom)
        {
            let key_ctrl = EventControllerKey::new();
            let sess = Rc::clone(&session);
            let area_clone = drawing_area.clone();

            key_ctrl.connect_key_pressed(move |_ctrl, key, _code, _mods| {
                let mut s = sess.borrow_mut();
                match key {
                    Key::v | Key::V => {
                        s.active_tool = NavigationTool::Move;
                    }
                    Key::b | Key::B => {
                        s.active_tool = NavigationTool::Brush;
                    }
                    Key::e | Key::E => {
                        s.active_tool = NavigationTool::Eraser;
                    }
                    Key::m | Key::M => {
                        s.active_tool = NavigationTool::Marquee;
                    }
                    Key::l | Key::L => {
                        s.active_tool = NavigationTool::Lasso;
                    }
                    Key::w | Key::W => {
                        s.active_tool = NavigationTool::Wand;
                    }
                    Key::c | Key::C => {
                        s.active_tool = NavigationTool::Crop;
                    }
                    Key::u | Key::U => {
                        s.active_tool = NavigationTool::Shape;
                    }
                    Key::t | Key::T => {
                        s.active_tool = NavigationTool::Type;
                    }
                    Key::i | Key::I => {
                        s.active_tool = NavigationTool::Eyedropper;
                    }
                    Key::h | Key::H => {
                        s.active_tool = NavigationTool::Hand;
                    }
                    Key::z | Key::Z => {
                        s.active_tool = NavigationTool::Zoom;
                    }
                    Key::x | Key::X => {
                        s.swap_colors();
                    }
                    Key::d | Key::D => {
                        s.reset_colors();
                    }
                    Key::bracketleft => {
                        s.brush_settings.size = (s.brush_settings.size - 5.0).max(1.0);
                    }
                    Key::bracketright => {
                        s.brush_settings.size = (s.brush_settings.size + 5.0).min(1000.0);
                    }
                    _ => return gtk4::glib::Propagation::Proceed,
                }
                area_clone.queue_draw();
                gtk4::glib::Propagation::Stop
            });

            drawing_area.add_controller(key_ctrl);
        }

        container.append(&drawing_area);

        Self {
            container,
            drawing_area,
            session,
            drag_start,
            last_mouse_pos,
            active_handle,
            on_document_modified,
        }
    }

    /// Converte coordenadas de tela para coordenadas do espaço do documento em pixels.
    fn screen_to_doc(
        session: &EditorSession,
        sx: f64,
        sy: f64,
        view_w: f64,
        view_h: f64,
    ) -> (f64, f64) {
        let doc_w = session
            .document
            .as_ref()
            .map(|d| d.width as f64)
            .unwrap_or(800.0);
        let doc_h = session
            .document
            .as_ref()
            .map(|d| d.height as f64)
            .unwrap_or(600.0);

        let center_x = view_w / 2.0 + session.pan_x * session.zoom;
        let center_y = view_h / 2.0 + session.pan_y * session.zoom;

        let doc_origin_x = center_x - (doc_w * session.zoom) / 2.0;
        let doc_origin_y = center_y - (doc_h * session.zoom) / 2.0;

        let doc_x = (sx - doc_origin_x) / session.zoom;
        let doc_y = (sy - doc_origin_y) / session.zoom;

        (doc_x, doc_y)
    }

    /// Desenha o canvas: fundo quadriculado, moldura do documento, camadas e contorno do cursor.
    fn draw_canvas(
        session: &EditorSession,
        cr: &Context,
        view_w: f64,
        view_h: f64,
        mouse_pos: (f64, f64),
    ) {
        // 1. Fundo da Área de Trabalho (Cinza Escuro)
        cr.set_source_rgb(0.18, 0.18, 0.18);
        cr.paint().ok();

        let doc = match &session.document {
            Some(d) => d,
            None => return,
        };

        let doc_w = doc.width as f64;
        let doc_h = doc.height as f64;

        let center_x = view_w / 2.0 + session.pan_x * session.zoom;
        let center_y = view_h / 2.0 + session.pan_y * session.zoom;

        let origin_x = center_x - (doc_w * session.zoom) / 2.0;
        let origin_y = center_y - (doc_h * session.zoom) / 2.0;

        // 2. Sombra e Moldura do Documento
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.4);
        cr.rectangle(
            origin_x + 4.0,
            origin_y + 4.0,
            doc_w * session.zoom,
            doc_h * session.zoom,
        );
        cr.fill().ok();

        // 3. Fundo Branco / Quadriculado do Documento
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(
            origin_x,
            origin_y,
            doc_w * session.zoom,
            doc_h * session.zoom,
        );
        cr.fill().ok();

        // 4. O canvas e a exportação usam o mesmo compositor de referência.
        cr.save().ok();
        cr.translate(origin_x, origin_y);
        cr.scale(session.zoom, session.zoom);
        if let Ok(composite) = crate::render::cpu_compositor::CpuCompositor::render(doc) {
            if let Ok(mut surface) =
                ImageSurface::create(Format::ARgb32, doc.width as i32, doc.height as i32)
            {
                let stride = surface.stride() as usize;
                {
                    let mut data = surface.data().expect("new Cairo surface must be writable");
                    for y in 0..doc.height {
                        let source_row =
                            &composite.pixels[y * doc.width * 4..(y + 1) * doc.width * 4];
                        let destination_row = &mut data[y * stride..y * stride + doc.width * 4];
                        for (src, dst) in source_row
                            .chunks_exact(4)
                            .zip(destination_row.chunks_exact_mut(4))
                        {
                            let alpha = src[3] as u16;
                            // Cairo ARgb32 no Linux little-endian é BGRA pré-multiplicado.
                            dst[0] = ((src[2] as u16 * alpha + 127) / 255) as u8;
                            dst[1] = ((src[1] as u16 * alpha + 127) / 255) as u8;
                            dst[2] = ((src[0] as u16 * alpha + 127) / 255) as u8;
                            dst[3] = src[3];
                        }
                    }
                }
                surface.mark_dirty();
                cr.set_source_surface(&surface, 0.0, 0.0).ok();
                cr.paint().ok();
            }
        }
        cr.restore().ok();

        // 5. Alças de Transformação da Camada Ativa (quando a ferramenta Mover estiver selecionada)
        if session.active_tool == NavigationTool::Move {
            if let Some(active_layer) = doc.active_layer() {
                let t = &active_layer.transform;
                let lx = origin_x + t.origin.x * session.zoom;
                let ly = origin_y + t.origin.y * session.zoom;
                let lw = t.size.width * session.zoom;
                let lh = t.size.height * session.zoom;

                // Caixa delimitadora
                cr.set_source_rgb(0.0, 0.48, 1.0);
                cr.set_line_width(1.5);
                cr.rectangle(lx, ly, lw, lh);
                cr.stroke().ok();

                // 8 Alças de controle
                cr.set_source_rgb(1.0, 1.0, 1.0);
                let handles = [
                    (lx, ly),
                    (lx + lw / 2.0, ly),
                    (lx + lw, ly),
                    (lx, ly + lh / 2.0),
                    (lx + lw, ly + lh / 2.0),
                    (lx, ly + lh),
                    (lx + lw / 2.0, ly + lh),
                    (lx + lw, ly + lh),
                ];
                for (hx, hy) in handles {
                    cr.rectangle(hx - 4.0, hy - 4.0, 8.0, 8.0);
                    cr.fill_preserve().ok();
                    cr.set_source_rgb(0.0, 0.48, 1.0);
                    cr.set_line_width(1.0);
                    cr.stroke().ok();
                    cr.set_source_rgb(1.0, 1.0, 1.0);
                }
            }
        }

        // 6. Indicador Circular do Pincel / Borracha sob o Cursor
        if session.active_tool == NavigationTool::Brush
            || session.active_tool == NavigationTool::Eraser
        {
            let radius = (session.brush_settings.size * session.zoom) / 2.0;
            cr.set_source_rgba(0.2, 0.2, 0.2, 0.8);
            cr.set_line_width(1.0);
            cr.arc(mouse_pos.0, mouse_pos.1, radius, 0.0, std::f64::consts::TAU);
            cr.stroke().ok();
        }
    }

    pub fn connect_document_modified<F: Fn() + 'static>(&self, f: F) {
        *self.on_document_modified.borrow_mut() = Some(Box::new(f));
    }
}
