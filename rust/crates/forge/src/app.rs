//! Application state and the per-frame loop. The UI only reads and writes
//! addresses on the bus and edits the rack model; everything audible or
//! routable lives in `rack-core`.

use crate::{drawer, matrix, rack, shell};
use egui::{Rect, Ui};
use rack_core::bus::Bus;
use rack_core::engine::Engine;
use rack_core::genesis::Genesis;
use rack_core::model::{self, CoreFrame, Instrument, RackState};
use rack_core::store::{self, Session, Snapshot, Vault, Workspace};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// The four apps of the launcher.
pub const APPS: [(&str, &str, &str); 4] = [
    ("forge", "forge", "FORGE UI DESIGNER"),
    ("perform", "perform", "PERFORMANCE · DJ"),
    ("rack", "rack", "MAIN RACK"),
    ("modular", "euro", "EURORACK · MODULAR"),
];

/// Forge stages: `(key, icon, name, tip)`.
pub const STAGES: [(&str, &str, &str, &str); 5] = [
    (
        "library",
        "library",
        "LIBRARY",
        "Every knob, fader, screen, glyph and composite",
    ),
    (
        "case",
        "modules",
        "MODULES",
        "Engine + utility modules → parameters",
    ),
    (
        "panel",
        "panel",
        "PANEL",
        "Lay out the face from library assets",
    ),
    (
        "layouts",
        "layouts",
        "LAYOUTS",
        "Partition templates, materials, resonance",
    ),
    (
        "graph",
        "graph",
        "NODE GRAPH",
        "Visual programming · Blueprint editing",
    ),
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SideState {
    pub tab: String,
    pub open: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub app: String,
    pub stage: String,
    pub sides: HashMap<String, SideState>,
    /// VIEW mode chosen per fill section (`device.slot` → mode).
    pub fills: HashMap<String, String>,
    pub targets: HashMap<String, bool>,
    pub drawer_open: bool,
    pub drawer_tab: String,
    /// Module scope: department index, or −1 for all.
    pub scope: i32,
    pub last_user: String,
}

impl Default for Prefs {
    fn default() -> Self {
        let targets = [
            ("narr", true),
            ("dmx", true),
            ("unreal", true),
            ("midi", false),
            ("ws", false),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
        Self {
            app: "forge".into(),
            stage: "library".into(),
            sides: HashMap::new(),
            fills: HashMap::new(),
            targets,
            drawer_open: false,
            drawer_tab: "console".into(),
            scope: -1,
            last_user: String::new(),
        }
    }
}

#[derive(Default)]
pub struct GateForm {
    pub user: String,
    pub conn: usize,
    pub host: String,
}

#[derive(Default)]
pub struct PickerForm {
    /// `(name, faction)` while the NEW VAULT card is being filled in.
    pub new: Option<(String, String)>,
    /// Vault id whose version history is expanded.
    pub hist: Option<String>,
    pub error: Option<String>,
}

/// What is being dragged between panels.
#[derive(Clone, Debug, PartialEq)]
pub enum DragItem {
    Inst(String),
    Fx(String),
    Device(String),
    Module(String),
}

/// Edits collected while drawing and applied after the frame's borrows end.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    AddDevice { inst: String, at: Option<usize> },
    MoveDevice { dev: String, at: usize },
    RemoveDevice(String),
    ToggleOpen(String),
    Select(String),
    AddFx { dev: String, fx: String },
    ToggleFx { dev: String, slot: usize },
    RemoveFx { dev: String, slot: usize },
    EditInstrument(String),
    NewInstrument(usize),
    FoldAll(bool),
    SetApp(&'static str),
    SetStage(&'static str),
    ToPicker,
    SignOut,
    SaveVersion,
    RestoreVersion(u32),
    SetConn(usize),
    Mount(Box<Genesis>),
}

pub struct ForgeApp {
    pub ws: Workspace,
    pub bus: Bus,
    pub engine: Engine,
    pub state: RackState,
    saved_state: RackState,
    pub genesis: Option<Genesis>,
    pub session: Option<Session>,
    pub vaults: Vec<Vault>,
    pub prefs: Prefs,
    saved_prefs: Prefs,
    pub core: CoreFrame,
    pub core_log: VecDeque<String>,
    pub sel: Option<String>,
    pub draft: Instrument,
    pub gate: GateForm,
    pub picker: PickerForm,
    pub actions: Vec<Action>,
    pub fps: f32,
    fps_n: u32,
    fps_t: f64,
    route_acc: f64,
    core_acc: f64,
    save_acc: f64,
    saved_rev: u64,
    log_step: i32,
}

impl ForgeApp {
    pub fn new(ctx: &egui::Context, root: std::path::PathBuf) -> Self {
        instrument_kit::theme::install(ctx);
        let ws = Workspace::new(root);
        let mut bus = Bus::new();
        Engine::seed(&mut bus);
        if let Some(params) = ws.load::<HashMap<String, rack_core::Value>>("params") {
            bus.restore(params);
        }
        let mut state = ws.load::<RackState>("rack").unwrap_or_default();
        state.merge_presets();
        let prefs = ws.load::<Prefs>("prefs").unwrap_or_default();
        let session = ws.load::<Session>("session");
        let vaults = store::load_vaults(&ws);
        let genesis = ws
            .load::<Genesis>("genesis")
            .or_else(|| Some(Genesis::oros_v7()));
        let mut engine = Engine::new();
        // A silent transport runs from the start so screens move before anything is armed.
        engine.play(&mut bus);
        let gate = GateForm {
            user: prefs.last_user.clone(),
            ..Default::default()
        };
        Self {
            saved_state: state.clone(),
            saved_prefs: prefs.clone(),
            saved_rev: bus.rev,
            ws,
            bus,
            engine,
            state,
            genesis,
            session,
            vaults,
            prefs,
            core: CoreFrame::default(),
            core_log: VecDeque::new(),
            sel: None,
            draft: Instrument::blank(0),
            gate,
            picker: PickerForm::default(),
            actions: vec![],
            fps: 0.0,
            fps_n: 0,
            fps_t: 0.0,
            route_acc: 0.0,
            core_acc: 0.0,
            save_acc: 0.0,
            log_step: -1,
        }
    }

    pub fn vault(&self) -> Option<&Vault> {
        let id = self.session.as_ref()?.vault.as_ref()?;
        self.vaults.iter().find(|v| &v.id == id)
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            rack: Some(self.state.clone()),
            params: self.bus.snapshot(),
            forge: HashMap::new(),
        }
    }

    fn apply_snapshot(&mut self, snap: &Snapshot) {
        if let Some(r) = &snap.rack {
            self.state = r.clone();
            self.state.merge_presets();
        }
        self.bus.restore(snap.params.clone());
        self.sel = None;
    }

    /// Open a vault: restore its working state (or a fresh rack for a new vault).
    pub fn open_vault(&mut self, id: &str, version: Option<u32>) {
        let Some(i) = self.vaults.iter().position(|v| v.id == id) else {
            return;
        };
        let snap = match version {
            Some(n) => self.vaults[i]
                .versions
                .iter()
                .find(|v| v.n == n)
                .map(|v| v.snap.clone()),
            None => self.vaults[i].work.clone(),
        };
        if let Some(s) = snap {
            self.apply_snapshot(&s);
        } else if self.vaults[i].fresh {
            self.state = RackState::factory();
            self.sel = None;
        }
        self.vaults[i].fresh = false;
        self.vaults[i].updated = store::now_ms();
        if let Some(s) = self.session.as_mut() {
            s.vault = Some(id.to_owned());
        }
        self.persist_session();
    }

    pub fn persist_session(&mut self) {
        match &self.session {
            Some(s) => {
                let _ = self.ws.save("session", s);
            }
            None => self.ws.remove("session"),
        }
        let _ = self.ws.save("vaults", &self.vaults);
    }

    fn apply(&mut self, a: Action) {
        match a {
            Action::AddDevice { inst, at } => {
                let at = at.unwrap_or(self.state.rack.len());
                if let Some(id) = self.state.add_device(&inst, at) {
                    self.sel = Some(id);
                }
            }
            Action::MoveDevice { dev, at } => self.state.move_device(&dev, at),
            Action::RemoveDevice(id) => {
                self.state.remove_device(&id);
                if self.sel.as_deref() == Some(id.as_str()) {
                    self.sel = None;
                }
            }
            Action::ToggleOpen(id) => {
                if let Some(d) = self.state.device_mut(&id) {
                    d.open = !d.open;
                }
            }
            Action::Select(id) => self.sel = Some(id),
            Action::AddFx { dev, fx } => {
                self.state.add_fx(&dev, &fx);
            }
            Action::ToggleFx { dev, slot } => {
                if let Some(f) = self.state.device_mut(&dev).and_then(|d| d.fx.get_mut(slot)) {
                    f.on = !f.on;
                }
            }
            Action::RemoveFx { dev, slot } => {
                if let Some(d) = self.state.device_mut(&dev) {
                    if slot < d.fx.len() {
                        d.fx.remove(slot);
                    }
                }
            }
            Action::EditInstrument(id) => {
                if let Some(i) = self.state.inst(&id) {
                    self.draft = i.clone();
                    self.prefs.app = "forge".into();
                    self.prefs.stage = "case".into();
                }
            }
            Action::NewInstrument(dept) => {
                self.draft = Instrument::blank(dept);
                self.prefs.app = "forge".into();
                self.prefs.stage = "case".into();
            }
            Action::FoldAll(open) => self.state.rack.iter_mut().for_each(|d| d.open = open),
            Action::SetApp(a) => self.prefs.app = a.into(),
            Action::SetStage(s) => {
                self.prefs.app = "forge".into();
                self.prefs.stage = s.into();
            }
            Action::ToPicker => {
                let snap = self.snapshot();
                if let Some(id) = self.session.as_ref().and_then(|s| s.vault.clone()) {
                    if let Some(v) = self.vaults.iter_mut().find(|v| v.id == id) {
                        v.work = Some(snap);
                        v.updated = store::now_ms();
                    }
                }
                if let Some(s) = self.session.as_mut() {
                    s.vault = None;
                }
                self.persist_session();
            }
            Action::SignOut => {
                self.session = None;
                self.persist_session();
            }
            Action::SaveVersion => {
                let snap = self.snapshot();
                if let Some(id) = self.session.as_ref().and_then(|s| s.vault.clone()) {
                    if let Some(v) = self.vaults.iter_mut().find(|v| v.id == id) {
                        v.save_version(snap);
                    }
                }
                self.persist_session();
            }
            Action::RestoreVersion(n) => {
                if let Some(id) = self.session.as_ref().and_then(|s| s.vault.clone()) {
                    self.open_vault(&id, Some(n));
                }
            }
            Action::SetConn(i) => {
                if let Some(s) = self.session.as_mut() {
                    s.conn = rack_core::catalog::CONNECTIONS[i.min(2)].0.into();
                }
                self.persist_session();
            }
            Action::Mount(g) => {
                let _ = self.ws.save("genesis", &*g);
                self.genesis = Some(*g);
            }
        }
    }

    fn tick(&mut self, dt: f64) {
        self.engine.update(&mut self.bus, dt);
        let t_ms = self.engine.time * 1000.0;

        self.route_acc += dt;
        if self.route_acc >= 0.05 {
            self.route_acc = 0.0;
            model::run_routes(&self.state, &mut self.bus, t_ms);
        }

        self.core_acc += dt;
        if self.core_acc >= 0.25 {
            self.core_acc = 0.0;
            self.core = model::core_frame(&self.state, &self.bus, t_ms);
            let step = self.bus.num("transport.step", 0.0) as i32;
            if step % 4 == 0 && step != self.log_step && !self.core.rows.is_empty() {
                let i = (self.engine.rand() * self.core.rows.len() as f32) as usize;
                let r = &self.core.rows[i.min(self.core.rows.len() - 1)];
                let ch =
                    rack_core::engine::ROMAN[(self.bus.num("rack.chapter", 0.0) as usize).min(11)];
                self.core_log.push_front(format!(
                    "ch {ch}·{}  {} {:.3}",
                    self.bus.num("rack.scene", 0.0) as usize + 1,
                    r.addr,
                    r.v
                ));
                self.core_log.truncate(6);
            }
            self.log_step = step;
        }

        self.fps_n += 1;
        self.fps_t += dt;
        if self.fps_t >= 0.5 {
            self.fps = self.fps_n as f32 / self.fps_t as f32;
            self.fps_n = 0;
            self.fps_t = 0.0;
        }

        // Autosave: model and preferences when they change, parameters at most once a second.
        self.save_acc += dt;
        if self.save_acc >= 1.0 {
            self.save_acc = 0.0;
            self.autosave();
        }
    }

    pub fn autosave(&mut self) {
        if self.state != self.saved_state && self.ws.save("rack", &self.state).is_ok() {
            self.saved_state = self.state.clone();
        }
        if self.prefs != self.saved_prefs && self.ws.save("prefs", &self.prefs).is_ok() {
            self.saved_prefs = self.prefs.clone();
        }
        if self.bus.rev != self.saved_rev && self.ws.save("params", &self.bus.snapshot()).is_ok() {
            self.saved_rev = self.bus.rev;
        }
    }

    /// Draw one frame into `ui`.
    pub fn ui(&mut self, ui: &mut Ui) {
        let dt = ui.input(|i| i.stable_dt) as f64;
        self.tick(dt);

        let full = ui.max_rect();
        ui.painter()
            .rect_filled(full, 0.0, instrument_kit::theme::SHELL);

        let has_user = self.session.as_ref().is_some_and(|s| !s.user.is_empty());
        let has_vault = self.session.as_ref().is_some_and(|s| s.vault.is_some());
        if !has_user {
            shell::gate(self, ui, full);
        } else if !has_vault {
            shell::vault_picker(self, ui, full);
        } else {
            self.workspace(ui, full);
        }

        for a in std::mem::take(&mut self.actions) {
            self.apply(a);
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    }

    fn workspace(&mut self, ui: &mut Ui, full: Rect) {
        let header_h = shell::HEADER_H;
        let (hd, mut body) = full.split_top_bottom_at_y(full.top() + header_h);
        let drawer_h = drawer::height(self);
        let dr = Rect::from_min_max(egui::pos2(body.left(), body.bottom() - drawer_h), body.max);
        body.max.y -= drawer_h;

        match self.prefs.app.as_str() {
            "rack" => rack::body(self, ui, body),
            "modular" => matrix::body(self, ui, body),
            "perform" => shell::stub(
                ui,
                body,
                "perform",
                "PERFORMANCE · DJ",
                "Live parameter tweaking across decks, crossfader and cue points.",
                "ARRIVES IN THE NEXT PASS · decks, mixer and Genesis discs",
            ),
            _ => crate::forge::body(self, ui, body),
        }
        drawer::show(self, ui, dr, body);
        // Header last so its menus sit above the workspace.
        shell::header(self, ui, hd);
    }
}
