mod bbcode;
mod embed;
mod parse;
mod search;
mod steam_news;
mod store;
mod synonyms;

use std::path::PathBuf;
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use crate::features::server_picker::ServerPickerState;
use crate::features::sync::LockExt;
pub use search::PatchSearchResult;
use store::Index;
pub use store::{PatchDetail, PatchOrigin, PatchSource};

const STEAM_NEWS_POLL: Duration = Duration::from_secs(15 * 60);

enum Job {
    New(Vec<(PatchSource, String)>),
    SteamNews(Vec<(PatchSource, String, Vec<String>)>),
}

/// Snapshot of an in-flight embedding batch, for the status bar and search UI. `done`/`total`/
/// `current_published` stay put after `indexing` drops back to false, so the last batch's numbers
/// remain visible until the next one starts.
#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct IndexingProgress {
    pub indexing: bool,
    pub done: usize,
    pub total: usize,
    /// ISO 8601 `published` of the patch currently being embedded.
    pub current_published: Option<String>,
}

#[derive(Default)]
pub struct PatchNotesState {
    index: Mutex<Option<Index>>,
    progress: Mutex<IndexingProgress>,
    /// Set once by `start`, from the dedicated `patch-notes-indexer` thread's setup. `ingest_new`
    /// and `ingest_steam_news` just enqueue onto it; the thread does the actual embedding.
    jobs: OnceLock<mpsc::Sender<Job>>,
}

impl PatchNotesState {
    fn path(app: &AppHandle) -> Option<PathBuf> {
        app.path().app_data_dir().ok().map(|d| d.join(store::FILE_NAME))
    }

    fn with_index<R>(&self, app: &AppHandle, f: impl FnOnce(&mut Index) -> R) -> R {
        let mut guard = self.index.lock_or_recover();
        let index = guard.get_or_insert_with(|| Self::path(app).map(|p| store::load(&p)).unwrap_or_default());
        f(index)
    }

    /// An owned copy, cheap enough for how rarely this runs (once per search), so the search
    /// itself can happen off the async runtime without holding a `State` borrow across `.await`.
    fn snapshot(&self, app: &AppHandle) -> Index {
        self.with_index(app, |index| Index { patches: index.patches.clone() })
    }

    fn persist(&self, app: &AppHandle) {
        let Some(path) = Self::path(app) else { return };
        let guard = self.index.lock_or_recover();
        if let Some(index) = guard.as_ref() {
            if let Err(e) = store::save(&path, index) {
                log::warn!("could not save {}: {e}", store::FILE_NAME);
            }
        }
    }

    /// Starts a new batch with a **known** size, computed up front by the caller (see
    /// `store::count_new_patches_lines`/`count_steam_news_lines`) before any embedding happens.
    /// This must know the real total before the first line embeds: incrementing `total` lazily,
    /// one line at a time as each embed starts, meant `done` could never trail `total` by more
    /// than 1 — for the common small-batch case (0-2 new lines per poll, once the initial backfill
    /// is done) the bar could only ever be observed at 0%, since `done` catching up to `total`
    /// happens in the same lock acquisition that flips `indexing` back to `false`. Only call this
    /// when `total > 0`; a zero-line batch does nothing, leaving the previous batch's numbers
    /// visible (see `indexing_progress`'s doc comment on `IndexingProgress`).
    fn begin_batch(&self, total: usize) {
        let mut progress = self.progress.lock_or_recover();
        progress.done = 0;
        progress.total = total;
        progress.indexing = true;
    }

    /// Marks which patch's line is about to be embedded, for the status bar's "currently indexing"
    /// date. The indexer thread processes one job at a time, so this never races `begin_batch`.
    fn begin_indexing(&self, source: &PatchSource) {
        self.progress.lock_or_recover().current_published = Some(source.published.clone());
    }

    fn advance_indexing(&self) {
        let mut progress = self.progress.lock_or_recover();
        progress.done += 1;
        if progress.done >= progress.total {
            progress.indexing = false;
        }
    }

    pub fn indexing_progress(&self) -> IndexingProgress {
        self.progress.lock_or_recover().clone()
    }

    fn enqueue(&self, job: Job) {
        match self.jobs.get() {
            Some(tx) if tx.send(job).is_ok() => {}
            Some(_) => log::warn!("patch notes indexer thread is gone; dropping a job"),
            None => log::warn!("patch notes indexer not started; dropping a job"),
        }
    }

    /// Hands `items` to the indexer thread; returns immediately.
    pub fn ingest_new(&self, items: Vec<(PatchSource, String)>) {
        self.enqueue(Job::New(items));
    }

    /// Hands a full Steam News fetch to the indexer thread; returns immediately.
    fn ingest_steam_news(&self, items: Vec<(PatchSource, String, Vec<String>)>) {
        self.enqueue(Job::SteamNews(items));
    }

    /// Indexes whatever in `items` is not already known. The index lock is held only to read
    /// `known` and to extend the index afterwards — never during embedding, which can take a
    /// while for a whole feed's worth of patches and must not block a concurrent search. Runs on
    /// the `patch-notes-indexer` thread only.
    fn process_new(&self, app: &AppHandle, items: Vec<(PatchSource, String)>) {
        let embedder = match embed::embedder() {
            Ok(e) => e,
            Err(e) => {
                log::warn!("skipping patch notes indexing: {e}");
                return;
            }
        };
        let known: std::collections::HashSet<String> =
            self.with_index(app, |index| index.patches.iter().map(|p| p.id.clone()).collect());
        let total = store::count_new_patches_lines(&items, &known);
        if total > 0 {
            self.begin_batch(total);
        }
        let new_patches = store::build_new_patches(
            &items,
            &known,
            embedder,
            |source| self.begin_indexing(source),
            || self.advance_indexing(),
        );
        if new_patches.is_empty() {
            return;
        }
        let added = new_patches.len();
        self.with_index(app, |index| index.patches.extend(new_patches));
        log::info!("patch notes index: {added} new patch(es) indexed");
        self.persist(app);
    }

    /// Reconciles a full Steam News fetch against the index: fills in patches that were only ever
    /// seen as a shallow forum-only entry, upgrades a patch whose `/v2/patches`-derived twin is
    /// still shallow, and adds anything neither feed has seen yet. See
    /// `plans/patch-notes-full-text.md` for why this needs its own fetch path. Runs on the
    /// `patch-notes-indexer` thread only.
    ///
    /// Works on a clone, same as `process_new`, so the index lock is held only to read the
    /// starting snapshot and to write the result back — never while `reconcile_steam_news` is
    /// embedding, which can take a long time when a lot of lines need it (e.g. after a change to
    /// how BBCode gets stripped invalidates many cached lines at once). This thread is the only
    /// writer to `index.patches` (jobs are processed one at a time from the same channel), so
    /// nothing else can race the clone-then-replace.
    fn process_steam_news(&self, app: &AppHandle, items: Vec<(PatchSource, String, Vec<String>)>) {
        let embedder = match embed::embedder() {
            Ok(e) => e,
            Err(e) => {
                log::warn!("skipping patch notes steam news reconciliation: {e}");
                return;
            }
        };
        let mut patches = self.with_index(app, |index| index.patches.clone());
        let total = store::count_steam_news_lines(&patches, &items);
        if total > 0 {
            self.begin_batch(total);
        }
        let changed = store::reconcile_steam_news(
            &mut patches,
            &items,
            embedder,
            |source| self.begin_indexing(source),
            || self.advance_indexing(),
        );
        if changed == 0 {
            return;
        }
        self.with_index(app, |index| index.patches = patches);
        log::info!("patch notes index: {changed} patch(es) added or upgraded from Steam News");
        self.persist(app);
    }

    async fn poll_steam_news(&self, app: &AppHandle) {
        let http = app.state::<ServerPickerState>().http.clone();
        let resp = match http.get(steam_news::URL).send().await.and_then(|r| r.error_for_status()) {
            Ok(resp) => resp,
            Err(e) => {
                log::warn!("steam news request failed: {e}");
                return;
            }
        };
        let text = match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                log::warn!("steam news body could not be read: {e}");
                return;
            }
        };
        let items = match steam_news::parse_news_with_text(&text) {
            Ok(items) => items,
            Err(e) => {
                log::warn!("steam news could not be parsed: {e}");
                return;
            }
        };
        self.ingest_steam_news(items);
    }
}

/// Starts everything patch notes needs for the rest of the app's life: the dedicated indexer
/// thread, and the Steam News poll loop that feeds it. Independent of `alerts::start`: `alerts`
/// keeps polling `/v2/patches` for its own card display, this polls the fuller Steam News source
/// on the same cadence to backfill and upgrade the index. Call once from `setup()`.
pub fn start(app: &AppHandle) {
    spawn_indexer_thread(app);

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            app.state::<PatchNotesState>().poll_steam_news(&app).await;
            tokio::time::sleep(STEAM_NEWS_POLL).await;
        }
    });
}

/// Spawns the thread that owns every bit of patch notes embedding work, for the app's whole
/// lifetime: it loads the ONNX model once, up front, then processes ingest jobs serially off the
/// async runtime as `alerts` and the Steam News poll hand them in. Named so it shows up as itself
/// rather than a generic worker in a profiler or task manager.
fn spawn_indexer_thread(app: &AppHandle) {
    let (tx, rx) = mpsc::channel::<Job>();
    if app.state::<PatchNotesState>().jobs.set(tx).is_err() {
        log::warn!("patch notes indexer already started; not spawning a second one");
        return;
    }

    let app = app.clone();
    std::thread::Builder::new()
        .name("patch-notes-indexer".into())
        .spawn(move || {
            let start = std::time::Instant::now();
            match embed::embedder() {
                Ok(_) => log::info!("patch notes search model ready in {:?}", start.elapsed()),
                Err(e) => log::warn!("patch notes search model failed to load: {e}"),
            }
            let state = app.state::<PatchNotesState>();
            for job in rx {
                match job {
                    Job::New(items) => state.process_new(&app, items),
                    Job::SteamNews(items) => state.process_steam_news(&app, items),
                }
            }
        })
        .expect("spawn patch-notes-indexer thread");
}

pub mod commands {
    use super::*;

    const RESULT_LIMIT: usize = 15;

    /// `async` so this runs off the main thread: Tauri runs a non-async command directly on it,
    /// and this can briefly contend the same index lock a long embedding pass holds.
    #[tauri::command]
    pub async fn patch_notes_indexing_progress(
        state: tauri::State<'_, PatchNotesState>,
    ) -> Result<IndexingProgress, ()> {
        Ok(state.indexing_progress())
    }

    /// Looks up one patch's full content for the in-app viewer, by the same id shown in the
    /// alerts list or a search result. `async` for the same reason as
    /// `patch_notes_indexing_progress` above — this must never be able to freeze the main thread.
    #[tauri::command]
    pub async fn get_patch_notes(
        app: AppHandle,
        state: tauri::State<'_, PatchNotesState>,
        id: String,
    ) -> Result<Option<PatchDetail>, ()> {
        Ok(state.with_index(&app, |index| index.patches.iter().find(|p| p.id == id).map(PatchDetail::from)))
    }

    /// Runs off the async runtime: loading/optimising the ONNX graph and running inference are
    /// both blocking CPU work.
    #[tauri::command]
    pub async fn search_patch_notes(
        app: AppHandle,
        state: tauri::State<'_, PatchNotesState>,
        query: String,
    ) -> Result<Vec<PatchSearchResult>, String> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let index = state.snapshot(&app);
        tauri::async_runtime::spawn_blocking(move || search::search(&index, &query, embed::embedder(), RESULT_LIMIT))
            .await
            .map_err(|e| e.to_string())
    }
}
