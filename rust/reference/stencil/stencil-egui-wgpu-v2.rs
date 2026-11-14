use eframe::{egui, egui_wgpu};
use std::sync::Arc;

// =========================================================================
// 1. Core Data Structures & Recursive BSP Layout Engine
// =========================================================================

/// A coordinate point on our infinite visual canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasPos {
    pub x: f32,
    pub y: f32,
}

/// Simple axis enum for splitting nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

/// The type of content that should be rendered inside a given leaf panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeafContentType {
    /// An infinite panning/zooming canvas workspace (Houdini-style)
    InfiniteCanvas,
    /// A static, form-filled inspector or sidebar panel
    StaticUiPanel,
}

/// Nodes in our layout tree.
#[derive(Debug, Clone, PartialEq)]
pub enum BspNode {
    Branch {
        axis: SplitAxis,
        ratio: f32,
        child_a: Box<BspNode>,
        child_b: Box<BspNode>,
    },
    Leaf {
        id: String,
        content_type: LeafContentType,
    },
}

// =========================================================================
// 2. Infinite Canvas Node & Zone Models
// =========================================================================

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub name: String,
    /// Position in local Canvas coordinates (not screen pixels!)
    pub canvas_pos: egui::Pos2,
    pub node_type: NodeType,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    /// A typical discrete data or compositor node
    Atomic,
    /// A visual grouping backdrop (similar to a network box in Houdini)
    Zone {
        size: egui::Vec2,
        color: egui::Color32,
    },
    /// A container node containing its own sub-graph
    Subnetwork {
        inner_nodes: Vec<Node>,
    },
}

// =========================================================================
// 3. Persistent UI/Application State
// =========================================================================

/// Tracks individual canvas projection and navigation paths.
pub struct InfiniteCanvasState {
    /// Scale factor: 1.0 = default, 0.1 = zoomed out, 5.0 = zoomed in
    pub zoom: f32,
    /// Offset translation in screen pixels from the panel's top-left corner
    pub pan: egui::Vec2,
    /// Track currently selected node
    pub selected_node_id: Option<String>,
    /// Houdini-style sub-network directory path (e.g. ["root", "vfx_layer", "particle_emitter"])
    pub directory_path: Vec<String>,
}

impl Default for InfiniteCanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::vec2(100.0, 100.0), // Center the default viewport slightly
            selected_node_id: None,
            directory_path: vec!["root".into()],
        }
    }
}

pub struct StencilApp {
    bsp_root: BspNode,
    custom_render_resources: Arc<CustomRenderResources>,
    /// The persistent state for our interactive infinite canvases.
    /// Since we can have multiple canvas panels, we track state with a hashmap or a primary state.
    primary_canvas_state: InfiniteCanvasState,
    /// Mock dataset of canvas nodes
    nodes: Vec<Node>,
}

struct CustomRenderResources {
    _pipeline: Option<wgpu::RenderPipeline>,
}

impl StencilApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let wgpu_state = cc.wgpu_render_state.as_ref().expect("WGPU render state is missing!");
        let _device = &wgpu_state.device;

        let resources = Arc::new(CustomRenderResources { _pipeline: None });

        // Instantiate nodes inside our mock editor canvas
        let mut nodes = vec![
            Node {
                id: "render_target".into(),
                name: "Primary Blend Node".into(),
                canvas_pos: egui::pos2(300.0, 200.0),
                node_type: NodeType::Atomic,
            },
            Node {
                id: "particle_stream".into(),
                name: "Boid Particles Layer".into(),
                canvas_pos: egui::pos2(100.0, 150.0),
                node_type: NodeType::Atomic,
            },
            Node {
                id: "effects_box".into(),
                name: "Post-Processing Group".into(),
                canvas_pos: egui::pos2(50.0, 80.0),
                node_type: NodeType::Zone {
                    size: egui::vec2(250.0, 200.0),
                    color: egui::Color32::from_rgba_unmultiplied(0, 150, 255, 15),
                },
            },
        ];

        // Let's nest a Subnetwork node inside
        nodes.push(Node {
            id: "deep_compositor".into(),
            name: "Hymn Subnet (Double-click)".into(),
            canvas_pos: egui::pos2(150.0, 320.0),
            node_type: NodeType::Subnetwork {
                inner_nodes: vec![
                    Node {
                        id: "sub_node_1".into(),
                        name: "Biometric Signal In".into(),
                        canvas_pos: egui::pos2(50.0, 50.0),
                        node_type: NodeType::Atomic,
                    }
                ],
            },
        });

        // Setup the BSP layout: Left 20% Sidebar UI, Right 80% Canvas editor
        let bsp_root = BspNode::Branch {
            axis: SplitAxis::Vertical,
            ratio: 0.2,
            child_a: Box::new(BspNode::Leaf {
                id: "side_controls".into(),
                content_type: LeafContentType::StaticUiPanel,
            }),
            child_b: Box::new(BspNode::Leaf {
                id: "infinite_workspace".into(),
                content_type: LeafContentType::InfiniteCanvas,
            }),
        };

        Self {
            bsp_root,
            custom_render_resources: resources,
            primary_canvas_state: InfiniteCanvasState::default(),
            nodes,
        }
    }

    // =========================================================================
    // 4. Matrix & Coordinate Projection Functions (Screen <-> Canvas)
    // =========================================================================

    /// Converts absolute screen coordinates to local, infinite canvas coordinates.
    pub fn screen_to_canvas(
        &self,
        screen_pos: egui::Pos2,
        viewport_rect: egui::Rect,
    ) -> egui::Pos2 {
        let zoom = self.primary_canvas_state.zoom;
        let pan = self.primary_canvas_state.pan;

        // 1. Translate relative to the viewport container bounds origin
        let relative_pos = screen_pos - viewport_rect.min;
        // 2. Subtract panning vector and divide by zoom factor
        egui::pos2(
            (relative_pos.x - pan.x) / zoom,
            (relative_pos.y - pan.y) / zoom,
        )
    }

    /// Converts local, infinite canvas coordinates to screen coordinates.
    pub fn canvas_to_screen(
        &self,
        canvas_pos: egui::Pos2,
        viewport_rect: egui::Rect,
    ) -> egui::Pos2 {
        let zoom = self.primary_canvas_state.zoom;
        let pan = self.primary_canvas_state.pan;

        // 1. Multiply by zoom and add panning vector
        // 2. Anchor back into screen-space origin of the viewport
        egui::pos2(
            viewport_rect.min.x + pan.x + (canvas_pos.x * zoom),
            viewport_rect.min.y + pan.y + (canvas_pos.y * zoom),
        )
    }

    // =========================================================================
    // 5. Recursive Layout Render Thread (Checks content_type)
    // =========================================================================

    fn render_bsp_node(&mut self, ui: &mut egui::Ui, node: &BspNode, rect: egui::Rect) {
        match node {
            BspNode::Leaf { id, content_type } => {
                // Isolate leaf rendering coordinates so elements do not bleed
                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.set_clip_rect(rect);

                    // Choose rendering engine depending on the leaf selection!
                    match content_type {
                        LeafContentType::InfiniteCanvas => {
                            self.render_infinite_canvas_leaf(ui, rect);
                        }
                        LeafContentType::StaticUiPanel => {
                            self.render_static_inspector_leaf(ui, id, rect);
                        }
                    }
                });
            }
            BspNode::Branch { axis, ratio, child_a, child_b } => {
                let clamped_ratio = ratio.clamp(0.0, 1.0);
                match axis {
                    SplitAxis::Vertical => {
                        let width_a = rect.width() * clamped_ratio;
                        let rect_a = egui::Rect::from_min_size(
                            rect.min,
                            egui::vec2(width_a, rect.height())
                        );
                        let rect_b = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + width_a, rect.min.y),
                            egui::vec2(rect.width() - width_a, rect.height())
                        );
                        self.render_bsp_node(ui, child_a, rect_a);
                        self.render_bsp_node(ui, child_b, rect_b);
                    }
                    SplitAxis::Horizontal => {
                        let height_a = rect.height() * clamped_ratio;
                        let rect_a = egui::Rect::from_min_size(
                            rect.min,
                            egui::vec2(rect.width(), height_a)
                        );
                        let rect_b = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x, rect.min.y + height_a),
                            egui::vec2(rect.width(), rect.height() - height_a)
                        );
                        self.render_bsp_node(ui, child_a, rect_a);
                        self.render_bsp_node(ui, child_b, rect_b);
                    }
                }
            }
        }
    }

    // =========================================================================
    // 6. Interactive Canvas Engine
    // =========================================================================

    fn render_infinite_canvas_leaf(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        // Allocate interactive viewport bounds
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

        // A. Panning (Middle Mouse Button / Scroll Wheel Press)
        if response.dragged_by(egui::PointerButton::Middle) {
            self.primary_canvas_state.pan += response.drag_delta();
        }

        // B. Zooming (Mouse Wheel)
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta);
            if scroll.y != 0.0 {
                if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let zoom_factor = 1.08_f32.powf(scroll.y / 50.0);
                    let anchor = pointer_pos - rect.min;

                    // Center zoom directly on pointer cursor
                    let prev_zoom = self.primary_canvas_state.zoom;
                    self.primary_canvas_state.zoom *= zoom_factor;
                    self.primary_canvas_state.zoom = self.primary_canvas_state.zoom.clamp(0.1, 5.0);

                    let actual_factor = self.primary_canvas_state.zoom / prev_zoom;
                    self.primary_canvas_state.pan = anchor - (anchor - self.primary_canvas_state.pan) * actual_factor;
                }
            }
        }

        // C. Left-Click (Selection and double-click check)
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(press_pos) = ui.input(|i| i.pointer.press_origin()) {
                let canvas_pos = self.screen_to_canvas(press_pos, rect);
                
                // Check if we hit any node (simple radius threshold)
                let mut hit_id = None;
                for node in &self.nodes {
                    let dist = (node.canvas_pos - canvas_pos).length();
                    if dist < 60.0 { // Simple radius selection boundary
                        hit_id = Some(node.id.clone());
                        break;
                    }
                }
                
                // Double-click to dive inside Subnetworks
                if let Some(ref hit) = hit_id {
                    let is_double_click = ui.input(|i| i.pointer.double_clicked_by(egui::PointerButton::Primary));
                    if is_double_click {
                        if let Some(node) = self.nodes.iter().find(|n| n.id == *hit) {
                            if let NodeType::Subnetwork { .. } = &node.node_type {
                                self.primary_canvas_state.directory_path.push(node.name.clone());
                                // In a production system, you'd replace 'self.nodes' with the subnet contents
                            }
                        }
                    }
                }

                self.primary_canvas_state.selected_node_id = hit_id;
            }
        }

        // D. Right-Click Quick Menu
        response.context_menu(|ui| {
            ui.label("Canvas Command Center");
            ui.separator();
            if ui.button("➕ Instantiate Logic Node").clicked() {
                // Instantiation event
                ui.close_menu();
            }
            if ui.button("📁 Encapsulate Subnet").clicked() {
                // Grouping event
                ui.close_menu();
            }
        });

        // E. Painting Canvas Artifacts (Visual Representation)
        let painter = ui.painter_at(rect);

        // Render an infinite coordinate dot grid background
        let grid_size = 50.0 * self.primary_canvas_state.zoom;
        let offset_x = self.primary_canvas_state.pan.x % grid_size;
        let offset_y = self.primary_canvas_state.pan.y % grid_size;

        let grid_color = egui::Color32::from_white_alpha(10);
        let mut x = rect.min.x + offset_x;
        while x < rect.max.x {
            let mut y = rect.min.y + offset_y;
            while y < rect.max.y {
                painter.circle_filled(egui::pos2(x, y), 1.0, grid_color);
                y += grid_size;
            }
            x += grid_size;
        }

        // First Draw Group Zones (as background layers)
        for node in &self.nodes {
            if let NodeType::Zone { size, color } = &node.node_type {
                let screen_min = self.canvas_to_screen(node.canvas_pos, rect);
                let scaled_size = *size * self.primary_canvas_state.zoom;
                let zone_rect = egui::Rect::from_min_size(screen_min, scaled_size);

                // Paint Backdrop Zone using pseudo glassmorphic effect
                painter.rect_filled(zone_rect, 6.0, *color);
                painter.rect_stroke(zone_rect, 6.0, egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)));
                painter.text(
                    screen_min + egui::vec2(10.0, 15.0),
                    egui::Align2::LEFT_CENTER,
                    &node.name,
                    egui::FontId::proportional(11.0 * self.primary_canvas_state.zoom),
                    egui::Color32::WHITE,
                );
            }
        }

        // Draw Nodes and Lines on Top
        for node in &self.nodes {
            if matches!(node.node_type, NodeType::Zone {..}) { continue; } // Already handled

            let screen_pos = self.canvas_to_screen(node.canvas_pos, rect);
            let radius = 24.0 * self.primary_canvas_state.zoom;
            let is_selected = Some(&node.id) == self.primary_canvas_state.selected_node_id.as_ref();

            let fill = if is_selected {
                egui::Color32::from_rgb(0, 120, 255)
            } else {
                egui::Color32::from_rgb(45, 45, 55)
            };

            // Main node circle
            painter.circle_filled(screen_pos, radius, fill);
            painter.circle_stroke(screen_pos, radius, egui::Stroke::new(1.5, egui::Color32::from_white_alpha(80)));

            // Text tag below
            painter.text(
                screen_pos + egui::vec2(0.0, radius + 15.0),
                egui::Align2::CENTER_CENTER,
                &node.name,
                egui::FontId::proportional(10.0 * self.primary_canvas_state.zoom),
                egui::Color32::WHITE,
            );
        }

        // Render Navigation Directory Header
        let dir_str = self.primary_canvas_state.directory_path.join(" / ");
        painter.text(
            rect.min + egui::vec2(15.0, 20.0),
            egui::Align2::LEFT_CENTER,
            format!("📍 {}", dir_str),
            egui::FontId::monospace(12.0),
            egui::Color32::from_rgb(0, 180, 255),
        );
    }

    // =========================================================================
    // 7. Static UI Layout Panel
    // =========================================================================

    fn render_static_inspector_leaf(&mut self, ui: &mut egui::Ui, id: &str, _rect: egui::Rect) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);
            ui.heading(format!("📁 {}", id.to_uppercase()));
            ui.separator();

            ui.add_space(4.0);
            ui.label("Layout Partition: STATIC");
            ui.add_space(8.0);

            ui.group(|ui| {
                ui.label("Selected Node Diagnostics");
                if let Some(selected_id) = &self.primary_canvas_state.selected_node_id {
                    ui.colored_label(egui::Color32::LIGHT_BLUE, format!("ID: {}", selected_id));
                    
                    if let Some(node) = self.nodes.iter_mut().find(|n| n.id == *selected_id) {
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut node.name);
                        });
                        
                        ui.label(format!("Canvas X: {:.1}", node.canvas_pos.x));
                        ui.label(format!("Canvas Y: {:.1}", node.canvas_pos.y));
                    }
                } else {
                    ui.weak("No node selected. Left-click a node on the canvas to inspect.");
                }
            });

            ui.add_space(12.0);

            ui.collapsing("Infinite Canvas Scaling Parameters", |ui| {
                ui.add(egui::Slider::new(&mut self.primary_canvas_state.zoom, 0.1..=5.0).text("Canvas Zoom"));
                ui.label(format!("Canvas Translation: x: {:.1}, y: {:.1}", self.primary_canvas_state.pan.x, self.primary_canvas_state.pan.y));
                if ui.button("🔄 Reset Pan/Zoom").clicked() {
                    self.primary_canvas_state = InfiniteCanvasState::default();
                }
            });
            
            ui.add_space(10.0);
            ui.weak("This leaf acts as a standard static container panel, completely isolated from coordinate-based scaling.");
        });
    }
}

// =========================================================================
// 8. Eframe Engine Hooks
// =========================================================================

impl eframe::App for StencilApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Upper menu bar
        egui::TopBottomPanel::top("designer_menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Export BSP Nodes").clicked() { /* ... */ }
                });
                ui.separator();
                ui.label("N-Layer Compositor Layout Designer");
            });
        });

        // Central Panel distributes space recursively through the BSP tree
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_rect = ui.available_rect_before_wrap();
            self.render_bsp_node(ui, &self.bsp_root, available_rect);
        });

        // Trigger continuous repaint to support visual physics, particles, and animations
        ctx.request_repaint();
    }
}
