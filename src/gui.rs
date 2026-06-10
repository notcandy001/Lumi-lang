// ============================================================
//  Lumi Language — GUI Backend  (egui / eframe)
//  Renders a Lumi component tree as a real native window.
// ============================================================

use eframe::egui;
use crate::interpreter::{ComponentInstance, Value};

// ── App state ────────────────────────────────────────────────

pub struct LumiApp {
    /// The root window component
    root: ComponentInstance,
    /// Mutable string state for `input` components (keyed by name)
    input_values: std::collections::HashMap<String, String>,
    /// Output log from print statements fired by event handlers
    event_log: Vec<String>,
}

impl LumiApp {
    pub fn new(root: ComponentInstance) -> Self {
        // Pre-populate input values from component defaults
        let mut input_values = std::collections::HashMap::new();
        collect_input_defaults(&root, &mut input_values);
        Self { root, input_values, event_log: Vec::new() }
    }
}

fn collect_input_defaults(
    comp: &ComponentInstance,
    map: &mut std::collections::HashMap<String, String>,
) {
    if comp.kind == "input" {
        let val = match comp.properties.get("value") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        map.insert(comp.name.clone(), val);
    }
    for child in &comp.children {
        collect_input_defaults(child, map);
    }
}

// ── eframe App impl ──────────────────────────────────────────

impl eframe::App for LumiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Get window title from root component
        let title = match self.root.properties.get("title") {
            Some(Value::String(s)) => s.clone(),
            _ => "Lumi App".to_string(),
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&title);
            ui.separator();

            // Clone children to avoid borrow issues while mutating input_values/event_log
            let children = self.root.children.clone();
            let mut log = std::mem::take(&mut self.event_log);
            let mut inputs = std::mem::take(&mut self.input_values);

            for child in &children {
                render_component(child, ui, &mut inputs, &mut log);
            }

            self.event_log = log;
            self.input_values = inputs;

            // Event log panel at the bottom
            if !self.event_log.is_empty() {
                ui.separator();
                ui.label("── Console ──");
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for line in &self.event_log {
                            ui.monospace(line);
                        }
                    });
            }
        });
    }
}

// ── Component renderer ───────────────────────────────────────

fn render_component(
    comp: &ComponentInstance,
    ui: &mut egui::Ui,
    inputs: &mut std::collections::HashMap<String, String>,
    log: &mut Vec<String>,
) {
    match comp.kind.as_str() {
        "layout" => render_layout(comp, ui, inputs, log),
        "text"   => render_text(comp, ui),
        "button" => render_button(comp, ui, inputs, log),
        "input"  => render_input(comp, ui, inputs),
        _        => {
            // Unknown component — render children recursively
            for child in &comp.children {
                render_component(child, ui, inputs, log);
            }
        }
    }
}

fn render_layout(
    comp: &ComponentInstance,
    ui: &mut egui::Ui,
    inputs: &mut std::collections::HashMap<String, String>,
    log: &mut Vec<String>,
) {
    let direction = match comp.properties.get("direction") {
        Some(Value::String(s)) => s.as_str() == "horizontal",
        _ => false,
    };
    let spacing = match comp.properties.get("spacing") {
        Some(Value::Number(n)) => *n as f32,
        _ => 8.0,
    };

    ui.spacing_mut().item_spacing.y = spacing;

    if direction {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = spacing;
            for child in &comp.children {
                render_component(child, ui, inputs, log);
            }
        });
    } else {
        ui.vertical(|ui| {
            for child in &comp.children {
                render_component(child, ui, inputs, log);
            }
        });
    }
}

fn render_text(comp: &ComponentInstance, ui: &mut egui::Ui) {
    let content = match comp.properties.get("content") {
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => String::new(),
    };
    let size = match comp.properties.get("size") {
        Some(Value::Number(n)) => *n as f32,
        _ => 14.0,
    };

    ui.label(egui::RichText::new(content).size(size));
}

fn render_button(
    comp: &ComponentInstance,
    ui: &mut egui::Ui,
    inputs: &mut std::collections::HashMap<String, String>,
    log: &mut Vec<String>,
) {
    let label = match comp.properties.get("text") {
        Some(Value::String(s)) => s.clone(),
        _ => comp.name.clone(),
    };

    if ui.button(&label).clicked() {
        // Fire all `on click` handlers
        for item in &comp.children {
            // ComponentInstance doesn't store event handlers directly —
            // they were executed at parse time. For now we store handler
            // output in the event log. In the future we'll store closures.
            let _ = item; // placeholder
        }
        // Log the click
        log.push(format!("[click] {}", comp.name));

        // Execute any print statements stored in click_output property
        // (set by the interpreter during component build)
        if let Some(Value::String(output)) = comp.properties.get("click_output") {
            for line in output.lines() {
                log.push(line.to_string());
            }
        }
    }

    let _ = inputs; // will be used for stateful buttons later
}

fn render_input(
    comp: &ComponentInstance,
    ui: &mut egui::Ui,
    inputs: &mut std::collections::HashMap<String, String>,
) {
    let placeholder = match comp.properties.get("placeholder") {
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };

    let value = inputs.entry(comp.name.clone()).or_default();
    let hint = egui::RichText::new(&placeholder).weak();

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&comp.name).weak());
        ui.add(
            egui::TextEdit::singleline(value)
                .hint_text(hint)
                .desired_width(200.0),
        );
    });
}

// ── Entry point ──────────────────────────────────────────────

pub fn run(root: ComponentInstance) -> Result<(), eframe::Error> {
    let width = match root.properties.get("width") {
        Some(Value::Number(n)) => *n as f32,
        _ => 800.0,
    };
    let height = match root.properties.get("height") {
        Some(Value::Number(n)) => *n as f32,
        _ => 600.0,
    };
    let title = match root.properties.get("title") {
        Some(Value::String(s)) => s.clone(),
        _ => "Lumi App".to_string(),
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_title(&title),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(|_cc| Box::new(LumiApp::new(root))),
    )
}
