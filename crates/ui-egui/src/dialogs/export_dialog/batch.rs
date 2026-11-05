//! A disk-file queue using exactly the same export pipeline as Export As.
//! Serializable view data stays in UiState; workers own snapshots and never touch documents.
#[cfg(not(target_arch = "wasm32"))]
use super::advanced;
use super::{Map, PhotocraftApp, Value, json};
use serde::{Deserialize, Serialize};

pub const COMMAND: &str = "file.export.batchConverter";
pub const MAX_FILES: usize = 10_000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub source: String,
    pub enabled: bool,
    pub status: String,
    pub output: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub visible: bool,
    pub entries: Vec<Entry>,
    pub fields: Map<String, Value>,
    pub filter: String,
    pub subfolders: bool,
    pub message: String,
    #[serde(skip)]
    pub running: bool,
    #[serde(skip)]
    pub progress: f32,
}

impl Default for State {
    fn default() -> Self {
        let mut fields = photocraft_engine::export_panel::AdvancedExport {
            full_size: true,
            resize_mode: "longEdge".into(),
            dont_enlarge: true,
            destination: "specificFolder".into(),
            ..Default::default()
        }
        .fields();
        fields.extend(
            json!({
                "format":"jxl", "quality":85, "lossless":false, "transparency":true,
                "scale":100, "resample":photocraft_engine::export_preset_cmds::DEFAULT_RESAMPLE,
                "metadata":"all", "profile":"document", "embedProfile":true,
                "sizing":"scale", "lockAspect":true, "tgaBits":"auto"
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        );
        Self { visible: false, entries: Vec::new(), fields, filter: "*".into(), subfolders: false, message: String::new(), running: false, progress: 0.0 }
    }
}

pub fn command(app: &mut PhotocraftApp, ctx: &egui::Context, params: &Value) -> Result<Value, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::command(app, ctx, params)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (app, ctx, params);
        Err("Batch Converter is available in the desktop app".into())
    }
}

pub fn tick(app: &mut PhotocraftApp, ctx: &egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    native::tick(app, ctx);
    #[cfg(target_arch = "wasm32")]
    let _ = (app, ctx);
}

pub fn show(app: &mut PhotocraftApp, ctx: &egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    native::show(app, ctx);
    #[cfg(target_arch = "wasm32")]
    let _ = (app, ctx);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native::Worker;

#[cfg(not(target_arch = "wasm32"))]
mod native;
