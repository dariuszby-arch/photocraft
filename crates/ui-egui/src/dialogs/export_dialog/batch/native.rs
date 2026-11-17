use super::{COMMAND, Entry, MAX_FILES, Map, PhotocraftApp, State, Value, advanced, json};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError},
    },
};

enum Event {
    Files(Vec<String>),
    Row(usize, Entry),
    Progress(f32, String),
    Error(String),
    Done,
}

pub(crate) struct Worker {
    receiver: Receiver<Event>,
    cancel: Arc<AtomicBool>,
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
    }
}

fn emit(tx: &SyncSender<Event>, ctx: &egui::Context, event: Event) {
    ctx.request_repaint();
    let _ = tx.send(event);
    ctx.request_repaint();
}

fn launch(
    app: &mut PhotocraftApp,
    ctx: &egui::Context,
    work: impl FnOnce(SyncSender<Event>, egui::Context, Arc<AtomicBool>) + Send + 'static,
) -> Result<(), String> {
    if app.batch_converter_worker.is_some() {
        return Err("Poczekaj na zakończenie bieżącej operacji".into());
    }
    let (tx, receiver) = mpsc::sync_channel(64);
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancel);
    let context = ctx.clone();
    std::thread::Builder::new()
        .name("photocraft-converter".into())
        .spawn(move || {
            let report = tx.clone();
            let wake = context.clone();
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| work(tx, context, flag))).is_err() {
                emit(&report, &wake, Event::Error("Nieoczekiwany błąd konwertera; pliki źródłowe pozostały bez zmian".into()));
            }
            emit(&report, &wake, Event::Done);
        })
        .map_err(|e| e.to_string())?;
    app.batch_converter_worker = Some(Worker { receiver, cancel });
    app.ui.batch_converter.running = true;
    app.ui.batch_converter.progress = 0.0;
    Ok(())
}

pub(super) fn tick(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let mut done = false;
    let mut files = None;
    if let Some(worker) = &app.batch_converter_worker {
        if !app.ui.batch_converter.visible {
            worker.cancel.store(true, Ordering::Release);
        }
        loop {
            match worker.receiver.try_recv() {
                Ok(Event::Files(paths)) => files = Some(paths),
                Ok(Event::Row(index, entry)) => {
                    if let Some(row) = app.ui.batch_converter.entries.get_mut(index) {
                        *row = entry;
                    }
                }
                Ok(Event::Progress(fraction, text)) => {
                    app.ui.batch_converter.progress = fraction.clamp(0.0, 1.0);
                    app.ui.batch_converter.message = text;
                }
                Ok(Event::Error(text)) => app.ui.batch_converter.message = text,
                Ok(Event::Done) | Err(TryRecvError::Disconnected) => {
                    done = true;
                    break;
                }
                Err(TryRecvError::Empty) => break,
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
    if done {
        app.batch_converter_worker = None;
        app.ui.batch_converter.running = false;
        for entry in &mut app.ui.batch_converter.entries {
            if entry.status == "W toku" {
                entry.status = "Przerwano".into();
            }
        }
    }
    if let Some(files) = files {
        match add_paths(&mut app.ui.batch_converter, files) {
            Ok(n) => app.ui.batch_converter.message = format!("Dodano {n} plików"),
            Err(e) => app.ui.batch_converter.message = e,
        }
    }
}

fn image_path(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    lower.ends_with(".xcf.gz")
        || path.extension().and_then(|s| s.to_str()).is_some_and(|ext| {
            let ext = ext.to_ascii_lowercase();
            ["pdn", "ora", "xcf", "xcfgz", "kra", "krz", "psd", "psb", "psdt", "svg", "svgz", "dng", "cr2", "nef", "nrw", "arw", "pef"].contains(&ext.as_str())
                || photocraft_codecs::from_extension(&ext).is_some_and(|f| f.caps().read)
        })
}

/// Greedy wildcard matching, bounded by filename length; '*' and '?' ignore letter case.
fn wildcard(name: &str, pattern: &str) -> bool {
    let text: Vec<char> = name.to_lowercase().chars().collect();
    let mask: Vec<char> = pattern.to_lowercase().chars().collect();
    let (mut i, mut j, mut star, mut resume) = (0, 0, None, 0);
    while i < text.len() {
        if mask.get(j).is_some_and(|c| *c == '?' || Some(c) == text.get(i)) {
            i += 1;
            j += 1;
        } else if mask.get(j) == Some(&'*') {
            star = Some(j);
            j += 1;
            resume = i;
        } else if let Some(at) = star {
            resume += 1;
            i = resume;
            j = at + 1;
        } else {
            return false;
        }
    }
    while mask.get(j) == Some(&'*') {
        j += 1;
    }
    j == mask.len()
}

fn matches_filter(path: &Path, filter: &str) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    filter.split([';', ',']).map(str::trim).filter(|p| !p.is_empty()).any(|p| {
        if p == "*.*" {
            return true;
        }
        let p = if !p.contains(['*', '?', '.']) {
            format!("*.{p}")
        } else if p.starts_with('.') {
            format!("*{p}")
        } else {
            p.into()
        };
        wildcard(name, &p)
    })
}

fn scan(folder: &Path, filter: &str, recursive: bool, output: &Path, cancel: &AtomicBool) -> Result<Vec<String>, String> {
    if filter.trim().is_empty() || filter.len() > 1024 {
        return Err("Podaj filtr, np. *.jpg;*.png;*.jxl".into());
    }
    let root = folder.canonicalize().map_err(|e| e.to_string())?;
    let excluded = output.canonicalize().ok().filter(|p| *p != root);
    let mut stack = vec![(root, 0usize)];
    let mut files = Vec::new();
    let mut visited = 0usize;
    while let Some((dir, depth)) = stack.pop() {
        if cancel.load(Ordering::Acquire) {
            return Err("Przerwano dodawanie katalogu".into());
        }
        for item in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            if cancel.load(Ordering::Acquire) {
                return Err("Przerwano dodawanie katalogu".into());
            }
            let item = item.map_err(|e| e.to_string())?;
            visited = visited.saturating_add(1);
            if visited > 200_000 {
                return Err("Katalog zawiera zbyt wiele pozycji; wybierz mniejszy katalog".into());
            }
            let ty = item.file_type().map_err(|e| e.to_string())?;
            let path = item.path();
            if ty.is_dir() && recursive && excluded.as_ref() != Some(&path) {
                if depth >= 64 {
                    return Err("Przekroczono 64 poziomy podkatalogów".into());
                }
                stack.push((path, depth + 1));
            } else if ty.is_file() && image_path(&path) && matches_filter(&path, filter) {
                files.push(path.to_str().ok_or("Nazwa pliku nie jest poprawnym tekstem Unicode")?.to_string());
                if files.len() > MAX_FILES {
                    return Err(format!("Jedna kolejka może zawierać do {MAX_FILES} plików"));
                }
            }
        }
    }
    files.sort_by_key(|s| s.to_lowercase());
    Ok(files)
}

fn add_paths(state: &mut State, paths: Vec<String>) -> Result<usize, String> {
    let mut seen: HashSet<PathBuf> = state.entries.iter().filter_map(|e| Path::new(&e.source).canonicalize().ok()).collect();
    let mut added = Vec::new();
    for source in paths {
        let path = Path::new(&source).canonicalize().map_err(|e| format!("{source}: {e}"))?;
        if !path.is_file() || !image_path(&path) {
            return Err(format!("Nieobsługiwany plik obrazu: {source}"));
        }
        if !seen.insert(path.clone()) {
            continue;
        }
        if state.entries.len().saturating_add(added.len()) >= MAX_FILES {
            return Err(format!("Jedna kolejka może zawierać do {MAX_FILES} plików"));
        }
        let source = path.to_str().ok_or("Nazwa pliku nie jest poprawnym tekstem Unicode")?.to_string();
        added.push(Entry { source, enabled: true, status: "Oczekuje".into(), output: String::new(), message: String::new() });
    }
    let n = added.len();
    state.entries.extend(added);
    Ok(n)
}

fn stop(app: &mut PhotocraftApp) {
    if let Some(worker) = &app.batch_converter_worker {
        worker.cancel.store(true, Ordering::Release);
        app.ui.batch_converter.message = "Zatrzymywanie…".into();
    }
}

fn start_scan(app: &mut PhotocraftApp, ctx: &egui::Context, folder: String) -> Result<(), String> {
    let state = &app.ui.batch_converter;
    let (filter, recursive) = (state.filter.clone(), state.subfolders);
    let output = PathBuf::from(super::super::s(&state.fields, "folder"));
    app.ui.batch_converter.message = "Odczytywanie katalogu…".into();
    launch(app, ctx, move |tx, ctx, cancel| match scan(Path::new(&folder), &filter, recursive, &output, &cancel) {
        Ok(files) => emit(&tx, &ctx, Event::Files(files)),
        Err(e) => emit(&tx, &ctx, Event::Error(e)),
    })
}

fn target_path(requested: &Path, policy: &str, sources: &HashSet<PathBuf>, written: &HashSet<PathBuf>) -> Result<Option<PathBuf>, String> {
    let parent = requested.parent().ok_or("Brak katalogu wynikowego")?.canonicalize().map_err(|e| e.to_string())?;
    let name = requested.file_name().ok_or("Brak nazwy wynikowej")?;
    let requested = parent.join(name);
    for number in 0..100_000usize {
        let candidate = if number == 0 {
            requested.clone()
        } else {
            let stem = requested.file_stem().and_then(|s| s.to_str()).ok_or("Nieprawidłowa nazwa wynikowa")?;
            let ext = requested.extension().and_then(|s| s.to_str()).unwrap_or("");
            parent.join(format!("{stem}_{number}.{ext}"))
        };
        let identity = candidate.canonicalize().unwrap_or_else(|_| candidate.clone());
        let protected = sources.contains(&identity) || written.contains(&identity);
        let exists = candidate.try_exists().map_err(|e| e.to_string())?;
        if !exists && !protected {
            return Ok(Some(candidate));
        }
        if policy == "skip" {
            return Ok(None);
        }
        if policy == "overwrite" {
            if protected {
                return Err("Plik wynikowy zastąpiłby źródło lub wcześniejszy wynik kolejki; wybierz dodawanie numeru".into());
            }
            return Ok(Some(candidate));
        }
    }
    Err("Nie można dobrać wolnej nazwy pliku".into())
}

struct Conversion<'a> {
    options: &'a photocraft_io::ImportOptions,
    sources: &'a HashSet<PathBuf>,
    written: &'a HashSet<PathBuf>,
    cancel: &'a AtomicBool,
}

fn convert_one(
    entry: &Entry,
    number: usize,
    fields: &Map<String, Value>,
    conversion: &Conversion<'_>,
    progress: &(dyn Fn(f32) + Sync),
) -> Result<Entry, String> {
    let Conversion { options, sources, written, cancel } = *conversion;
    let check = || if cancel.load(Ordering::Acquire) { Err("Przerwano".to_string()) } else { Ok(()) };
    check()?;
    let mut f = fields.clone();
    f.insert("__sourcePath".into(), json!(entry.source));
    let cancelled = || cancel.load(Ordering::Acquire);
    let importing = |v| progress(v * 0.12);
    let ctl = photocraft_raster::Interrupt::new(&cancelled, &importing);
    let imported = photocraft_io::import_file(&entry.source, Path::new(&entry.source), options, &ctl).map_err(|e| e.to_string())?;
    if imported.preview_only {
        return Err("Dostępny jest tylko podgląd tego pliku; konwersja pełnego obrazu jest niemożliwa".into());
    }
    check()?;
    let doc = imported.document;
    let name = super::super::export_filename(&doc, &f, number)?;
    let requested = advanced::destination(&f, &name)?.ok_or("Wybierz katalog wynikowy")?;
    let parent = Path::new(&requested).parent().ok_or("Brak katalogu wynikowego")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let a = advanced::read(&f)?;
    let Some(target) = target_path(Path::new(&requested), &a.conflict, sources, written)? else {
        return Ok(Entry { status: "Pominięto".into(), output: requested, message: "Plik już istnieje".into(), ..entry.clone() });
    };
    let target_text = target.to_str().ok_or("Nieprawidłowa ścieżka wynikowa")?;
    let source_profile = photocraft_engine::color_cmds::document_profile(&doc);
    let prepared = super::super::export_document(&doc, &f, None, &source_profile)?;
    check()?;
    progress(0.20);
    let mut result = advanced::encode(&prepared, &f, target_text, |d, p, settings| {
        check()?;
        let encoding = |v| progress(0.20 + v * 0.65);
        let ctl = photocraft_raster::Interrupt::new(&cancelled, &encoding);
        photocraft_io::export_with(d, p, &settings.io_options(), &ctl).map(|r| (r.bytes, r.warnings)).map_err(|e| e.to_string())
    })?;
    check()?;
    progress(0.90);
    // All inputs, including unchecked ones, are protected; only a complete encode is published.
    photocraft_format::atomic::atomic_write(&target, &result.bytes).map_err(|e| e.to_string())?;
    result.warnings.extend(imported.warnings);
    progress(1.0);
    Ok(Entry { status: "Gotowe".into(), output: target_text.into(), message: result.warnings.join("; "), ..entry.clone() })
}

fn start(app: &mut PhotocraftApp, ctx: &egui::Context) -> Result<(), String> {
    let state = &app.ui.batch_converter;
    if !state.entries.iter().any(|e| e.enabled) {
        return Err("Zaznacz pliki do konwersji".into());
    }
    let f = state.fields.clone();
    for (key, value) in &f {
        if !super::super::valid_setting(key, value) {
            return Err(format!("Nieprawidłowe ustawienie: {key}"));
        }
    }
    let fmt = super::super::s_fmt(&f);
    if fmt == "original" {
        return Err("Wybierz format konwersji".into());
    }
    let advanced = advanced::read(&f)?;
    if advanced.destination != "specificFolder" || !Path::new(advanced.folder.trim()).is_absolute() {
        return Err("Podaj pełną ścieżkę katalogu wynikowego".into());
    }
    let mut entries = state.entries.clone();
    if app.automation_input {
        let authorize = app.services.automation_command.as_ref().ok_or("Batch conversion requires authorized file access")?;
        for entry in entries.iter().filter(|e| e.enabled) {
            authorize("file.open", &json!({"path":entry.source}))?;
        }
        // The native converter uses a directory and generated filenames, so an automation must
        // authorize its own batch command before dispatch (the desktop defaults to deny).
        authorize(COMMAND, &json!({"input":entries, "output":advanced.folder}))?;
    }
    let sources = entries.iter().map(|e| Path::new(&e.source).canonicalize().unwrap_or_else(|_| PathBuf::from(&e.source))).collect::<HashSet<_>>();
    let options = photocraft_engine::jobs::import_options(app.session.prefs());
    for entry in entries.iter_mut().filter(|e| e.enabled) {
        entry.status = "Oczekuje".into();
        entry.output.clear();
        entry.message.clear();
    }
    app.ui.batch_converter.entries = entries.clone();
    launch(app, ctx, move |tx, ctx, cancel| {
        let total = entries.iter().filter(|e| e.enabled).count();
        let (mut done, mut ok, mut failed, mut skipped) = (0usize, 0usize, 0usize, 0usize);
        let mut written = HashSet::new();
        for (index, entry) in entries.iter().enumerate().filter(|(_, e)| e.enabled) {
            if cancel.load(Ordering::Acquire) {
                break;
            }
            emit(&tx, &ctx, Event::Row(index, Entry { status: "W toku".into(), ..entry.clone() }));
            let label = Path::new(&entry.source).file_name().and_then(|n| n.to_str()).unwrap_or(&entry.source).to_string();
            let highest = std::sync::atomic::AtomicU32::new(0);
            let progress = |v: f32| {
                // A size limit may encode several qualities. Keep progress monotonic and bound
                // UI messages to 200 per file even when a decoder reports every scanline.
                if !v.is_finite() {
                    return;
                }
                let step = (v.clamp(0.0, 1.0) * 200.0) as u32;
                if step <= highest.fetch_max(step, Ordering::Relaxed) {
                    return;
                }
                emit(&tx, &ctx, Event::Progress((done as f32 + step as f32 / 200.0) / total as f32, format!("{} / {total} — {label}", done + 1)));
            };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                convert_one(entry, done + 1, &f, &Conversion { options: &options, sources: &sources, written: &written, cancel: &cancel }, &progress)
            }))
            .unwrap_or_else(|_| Err("Nieoczekiwany błąd odczytu lub kodowania obrazu".into()));
            if cancel.load(Ordering::Acquire) && result.is_err() {
                emit(&tx, &ctx, Event::Row(index, Entry { status: "Przerwano".into(), ..entry.clone() }));
                break;
            }
            let row = match result {
                Ok(row) => {
                    if row.status == "Gotowe" {
                        ok += 1;
                        written.insert(PathBuf::from(&row.output));
                    } else {
                        skipped += 1;
                    }
                    row
                }
                Err(message) => {
                    failed += 1;
                    Entry { status: "Błąd".into(), message, ..entry.clone() }
                }
            };
            done += 1;
            emit(&tx, &ctx, Event::Row(index, row));
        }
        let prefix = if done < total { "Przerwano" } else { "Zakończono" };
        emit(
            &tx,
            &ctx,
            Event::Progress(done as f32 / total as f32, format!("{prefix}: {ok} gotowych, {skipped} pominiętych, {failed} błędów ({done}/{total})")),
        );
    })
}

pub(super) fn command(app: &mut PhotocraftApp, ctx: &egui::Context, params: &Value) -> Result<Value, String> {
    app.ui.batch_converter.visible = true;
    let action = params.get("action").and_then(Value::as_str).unwrap_or("open");
    if action == "cancel" {
        stop(app);
        return Ok(json!({"cancelRequested":true}));
    }
    if action != "open" && app.batch_converter_worker.is_some() {
        return Err("Konwerter jest zajęty".into());
    }
    match action {
        "open" => {}
        "add" => {
            let paths = params
                .get("files")
                .and_then(Value::as_array)
                .ok_or("Podaj tablicę files")?
                .iter()
                .map(|v| v.as_str().map(str::to_string).ok_or("Ścieżka musi być tekstem".to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            return add_paths(&mut app.ui.batch_converter, paths).map(|n| json!({"added":n}));
        }
        "folder" => {
            let folder = params.get("path").and_then(Value::as_str).ok_or("Podaj path katalogu")?;
            if let Some(filter) = params.get("filter").and_then(Value::as_str) {
                app.ui.batch_converter.filter = filter.into();
            }
            if let Some(value) = params.get("subfolders").and_then(Value::as_bool) {
                app.ui.batch_converter.subfolders = value;
            }
            start_scan(app, ctx, folder.into())?;
        }
        "start" => {
            if let Some(fields) = params.get("settings").and_then(Value::as_object) {
                app.ui.batch_converter.fields.extend(fields.clone());
            }
            start(app, ctx)?;
        }
        _ => return Err("Nieznana czynność konwertera".into()),
    }
    Ok(json!({"visible":true,"running":app.ui.batch_converter.running}))
}

fn choose_files(app: &mut PhotocraftApp, multiple: bool) -> Result<Value, String> {
    let request = crate::FileDialogRequest::Open { multiple, initial_dir: None, extensions: None };
    app.ask_file(request, |app, answer| match answer {
        crate::FileDialogAnswer::Paths(paths) => {
            if app.batch_converter_worker.is_some() {
                return Err("Konwerter jest zajęty".into());
            }
            let added = add_paths(&mut app.ui.batch_converter, paths)?;
            Ok(json!({"added":added}))
        }
        _ => Err("Wybierz obrazy na dysku".into()),
    })
}

pub(super) fn show(app: &mut PhotocraftApp, ctx: &egui::Context) {
    tick(app, ctx);
    if !app.ui.batch_converter.visible {
        return;
    }
    let mut state = std::mem::take(&mut app.ui.batch_converter);
    let mut visible = true;
    let mut action = "";
    let size = (ctx.content_rect().size() - egui::vec2(40.0, 80.0)).min(egui::vec2(1120.0, 720.0));
    egui::Window::new("PhotoCraft — konwersja wielu zdjęć")
        .id(egui::Id::unique(COMMAND))
        .open(&mut visible)
        .default_size(size)
        .max_size(size)
        .min_size(egui::vec2(680.0, 420.0))
        .show(ctx, |ui| {
            ui.label("Jeden format i wspólne ustawienia dla wszystkich zaznaczonych plików.");
            ui.separator();
            let height = (size.y - 225.0).max(220.0);
            ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), height), egui::Layout::left_to_right(egui::Align::Min), |ui| {
                let left = (ui.available_width() * 0.57).max(340.0);
                ui.allocate_ui_with_layout(egui::vec2(left, height), egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.add_enabled_ui(!state.running, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Dodaj plik…").clicked() {
                                action = "file";
                            }
                            if ui.button("Dodaj pliki…").clicked() {
                                action = "files";
                            }
                            if ui.button("Dodaj katalog…").clicked() {
                                action = "folder";
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Filtr katalogu:");
                            ui.add(egui::TextEdit::singleline(&mut state.filter).desired_width(200.0))
                                .on_hover_text("Np. *.jpg;*.png;*.jxl albo wakacje*.jpg. Wielkość liter nie ma znaczenia.");
                        });
                        ui.checkbox(&mut state.subfolders, "Uwzględnij podkatalogi");
                        ui.horizontal_wrapped(|ui| {
                            if ui.small_button("Wszystkie").clicked() {
                                for e in &mut state.entries {
                                    e.enabled = true;
                                }
                            }
                            if ui.small_button("Żadne").clicked() {
                                for e in &mut state.entries {
                                    e.enabled = false;
                                }
                            }
                            if ui.small_button("Usuń zaznaczone").clicked() {
                                state.entries.retain(|e| !e.enabled);
                            }
                            if ui.small_button("Wyczyść listę").clicked() {
                                state.entries.clear();
                            }
                        });
                    });
                    let selected = state.entries.iter().filter(|e| e.enabled).count();
                    ui.small(format!("Plików: {} | Do konwersji: {selected}", state.entries.len()));
                    let list_height = (height - 180.0).max(100.0);
                    egui::ScrollArea::vertical()
                        .id_salt("converter-files")
                        .max_height(list_height)
                        .min_scrolled_height(list_height)
                        .auto_shrink([false, false])
                        .show_rows(ui, 26.0, state.entries.len(), |ui, range| {
                            for index in range {
                                let Some(entry) = state.entries.get_mut(index) else { continue };
                                ui.horizontal(|ui| {
                                    ui.add_enabled(!state.running, egui::Checkbox::without_text(&mut entry.enabled));
                                    let name = Path::new(&entry.source).file_name().and_then(|n| n.to_str()).unwrap_or(&entry.source);
                                    ui.allocate_ui_with_layout(
                                        egui::vec2((left - 145.0).max(150.0), 24.0),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| ui.add(egui::Label::new(name).truncate()),
                                    )
                                    .inner
                                    .on_hover_text(format!("{}\n{}\n{}", entry.source, entry.output, entry.message));
                                    ui.label(&entry.status).on_hover_text(&entry.message);
                                });
                            }
                        });
                    ui.add_enabled_ui(!state.running, |ui| {
                        ui.label("Katalog wynikowy:");
                        ui.horizontal(|ui| {
                            let mut folder = super::super::s(&state.fields, "folder");
                            ui.add(egui::TextEdit::singleline(&mut folder).desired_width((left - 95.0).max(200.0)));
                            state.fields.insert("folder".into(), json!(folder));
                            if ui.button("Wybierz…").clicked() {
                                action = "output";
                            }
                        });
                    });
                });
                ui.separator();
                let right = ui.available_width().max(290.0);
                egui::ScrollArea::vertical().id_salt("converter-settings").max_height(height).min_scrolled_height(height).auto_shrink([false, false]).show(
                    ui,
                    |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(right);
                            ui.add_enabled_ui(!state.running, |ui| {
                                super::super::advanced::batch_controls(app, ui, &mut state.fields);
                            });
                        });
                    },
                );
            });
            ui.separator();
            ui.add(egui::ProgressBar::new(state.progress).show_percentage().desired_width(ui.available_width()));
            ui.horizontal(|ui| {
                if state.running {
                    ui.spinner();
                }
                ui.add(egui::Label::new(&state.message).wrap());
            });
            ui.horizontal(|ui| {
                if ui.add_enabled(!state.running && state.entries.iter().any(|e| e.enabled), egui::Button::new("Rozpocznij konwersję")).clicked() {
                    action = "start";
                }
                if ui.add_enabled(state.running, egui::Button::new("Zatrzymaj")).clicked() {
                    action = "stop";
                }
                if ui.button("Zamknij").clicked() {
                    action = "close";
                }
            });
        });
    state.visible = visible && action != "close";
    app.ui.batch_converter = state;
    let result = match action {
        "file" => choose_files(app, false),
        "files" => choose_files(app, true),
        "folder" => {
            let ctx = ctx.clone();
            app.ask_folder(move |app, folder| start_scan(app, &ctx, folder).map(|_| Value::Null))
        }
        "output" => app.ask_folder(|app, folder| {
            app.ui.batch_converter.fields.insert("folder".into(), json!(folder));
            Ok(Value::Null)
        }),
        "start" => start(app, ctx).map(|_| Value::Null),
        "stop" | "close" => {
            stop(app);
            Ok(Value::Null)
        }
        _ => Ok(Value::Null),
    };
    if let Err(e) = result {
        app.ui.batch_converter.message = e;
    }
    if !app.ui.batch_converter.visible {
        stop(app);
    }
}

#[cfg(test)]
mod tests;
