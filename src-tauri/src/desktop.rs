use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewWindow};

#[cfg(target_os = "macos")]
mod native;

pub const NOTCH_LABEL: &str = "notch";
pub const DRAWER_STATE_EVENT: &str = "gitnotch://drawer-state";

const RIBBON_WIDTH: u32 = 28;
const RIBBON_HEIGHT: u32 = 112;
const DRAWER_WIDTH: u32 = 960;
const DRAWER_MAX_HEIGHT: u32 = 600;
const DRAWER_EDGE_MARGIN: u32 = 24;
const DRAWER_HEIGHT_MARGIN: u32 = 24;
#[cfg(target_os = "macos")]
const OPEN_MS: u64 = 380;
#[cfg(target_os = "macos")]
const CLOSE_MS: u64 = 240;
const HOVER_DWELL: Duration = Duration::from_millis(120);
const HOVER_EXIT_TOLERANCE: Duration = Duration::from_millis(250);
const HOVER_POLL: Duration = Duration::from_millis(60);
const HOVER_MARGIN: u32 = 12;
const RIBBON_RADIUS: f64 = 12.0;
const DRAWER_RADIUS: f64 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DrawerIntent {
    Closed,
    Preview,
    Pinned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DrawerPhase {
    Resting,
    Opening,
    Closing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopView {
    pub intent: DrawerIntent,
    pub phase: DrawerPhase,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InteractionGuards {
    pub selection: bool,
    pub pointer_capture: bool,
}

impl InteractionGuards {
    fn blocks_auto_collapse(self) -> bool {
        self.selection || self.pointer_capture
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerAction {
    Transition {
        view: DesktopView,
        form: FormState,
        focus: bool,
    },
    State {
        view: DesktopView,
        focus: bool,
    },
    Unchanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormState {
    Closed,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoundedCorners {
    Left,
    All,
}

#[derive(Debug)]
pub struct DesktopState {
    intent: DrawerIntent,
    phase: DrawerPhase,
    generation: u64,
    inside_since: Option<Instant>,
    outside_since: Option<Instant>,
    suppress_hover: bool,
    interaction_guards: InteractionGuards,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopState {
    pub fn new() -> Self {
        Self {
            intent: DrawerIntent::Closed,
            phase: DrawerPhase::Resting,
            generation: 1,
            inside_since: None,
            outside_since: None,
            suppress_hover: false,
            interaction_guards: InteractionGuards::default(),
        }
    }

    pub fn view(&self) -> DesktopView {
        DesktopView {
            intent: self.intent,
            phase: self.phase,
            generation: self.generation,
        }
    }

    pub fn pointer(&mut self, inside: bool, moved: bool, now: Instant) -> DrawerAction {
        if inside {
            self.outside_since = None;

            if self.suppress_hover {
                self.inside_since = None;
                return DrawerAction::Unchanged;
            }

            if self.intent != DrawerIntent::Closed {
                self.inside_since = None;
                return DrawerAction::Unchanged;
            }

            if !moved && self.inside_since.is_none() {
                return DrawerAction::Unchanged;
            }

            let since = *self.inside_since.get_or_insert(now);
            if now.saturating_duration_since(since) < HOVER_DWELL {
                return DrawerAction::Unchanged;
            }

            self.inside_since = None;
            self.transition(DrawerIntent::Preview, FormState::Open, false)
        } else {
            self.inside_since = None;
            self.suppress_hover = false;

            if self.intent != DrawerIntent::Preview
                || self.interaction_guards.blocks_auto_collapse()
            {
                self.outside_since = None;
                return DrawerAction::Unchanged;
            }

            let since = *self.outside_since.get_or_insert(now);
            if now.saturating_duration_since(since) < HOVER_EXIT_TOLERANCE {
                return DrawerAction::Unchanged;
            }

            self.outside_since = None;
            self.close()
        }
    }

    pub fn toggle(&mut self) -> DrawerAction {
        match self.intent {
            DrawerIntent::Closed => self.transition(DrawerIntent::Pinned, FormState::Open, true),
            DrawerIntent::Preview => {
                if self.phase == DrawerPhase::Opening {
                    self.transition(DrawerIntent::Pinned, FormState::Open, true)
                } else {
                    self.intent = DrawerIntent::Pinned;
                    self.generation += 1;
                    DrawerAction::State {
                        view: self.view(),
                        focus: true,
                    }
                }
            }
            DrawerIntent::Pinned => {
                self.suppress_hover = true;
                self.close()
            }
        }
    }

    pub fn collapse(&mut self) -> DrawerAction {
        match self.intent {
            DrawerIntent::Closed => DrawerAction::Unchanged,
            DrawerIntent::Preview | DrawerIntent::Pinned => {
                self.suppress_hover = true;
                self.close()
            }
        }
    }

    pub fn blur(&mut self) -> DrawerAction {
        if self.intent == DrawerIntent::Preview && !self.interaction_guards.blocks_auto_collapse() {
            self.suppress_hover = true;
            self.close()
        } else {
            DrawerAction::Unchanged
        }
    }

    pub fn set_interaction_guards(&mut self, interaction_guards: InteractionGuards) {
        self.interaction_guards = interaction_guards;
        if interaction_guards.blocks_auto_collapse() {
            self.outside_since = None;
        }
    }

    pub fn complete_transition(&mut self, generation: u64) -> Option<DesktopView> {
        if self.generation != generation || self.phase == DrawerPhase::Resting {
            return None;
        }

        self.phase = DrawerPhase::Resting;
        Some(self.view())
    }

    fn transition(&mut self, intent: DrawerIntent, form: FormState, focus: bool) -> DrawerAction {
        self.intent = intent;
        self.phase = match form {
            FormState::Closed => DrawerPhase::Closing,
            FormState::Open => DrawerPhase::Opening,
        };
        self.generation += 1;
        DrawerAction::Transition {
            view: self.view(),
            form,
            focus,
        }
    }

    fn close(&mut self) -> DrawerAction {
        self.transition(DrawerIntent::Closed, FormState::Closed, false)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopCapabilities {
    pub platform: &'static str,
    pub work_area: Option<Rect>,
    pub scale_factor: Option<f64>,
    pub drawer: DesktopView,
    pub material: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DesktopAppearance {
    pub material: &'static str,
}

#[tauri::command]
pub fn toggle_drawer(
    app: AppHandle,
    state: State<'_, Mutex<DesktopState>>,
) -> Result<DesktopView, String> {
    let (action, view) = {
        let mut guard = state
            .lock()
            .map_err(|error| format!("Falha de sincronização interna: {error}"))?;
        let action = guard.toggle();
        (action, guard.view())
    };

    apply(&app, action)?;
    Ok(view)
}

#[tauri::command]
pub fn collapse_drawer(
    app: AppHandle,
    state: State<'_, Mutex<DesktopState>>,
) -> Result<DesktopView, String> {
    let (action, view) = {
        let mut guard = state
            .lock()
            .map_err(|error| format!("Falha de sincronização interna: {error}"))?;
        let action = guard.collapse();
        (action, guard.view())
    };

    apply(&app, action)?;
    Ok(view)
}

#[tauri::command]
pub fn set_drawer_interaction(
    interaction_guards: InteractionGuards,
    state: State<'_, Mutex<DesktopState>>,
) -> Result<DesktopView, String> {
    let mut guard = state
        .lock()
        .map_err(|error| format!("Falha de sincronização interna: {error}"))?;
    guard.set_interaction_guards(interaction_guards);
    Ok(guard.view())
}

#[tauri::command]
pub fn get_desktop_capabilities(
    app: AppHandle,
    state: State<'_, Mutex<DesktopState>>,
) -> Result<DesktopCapabilities, String> {
    let drawer = state
        .lock()
        .map_err(|error| format!("Falha de sincronização interna: {error}"))?
        .view();
    let monitor = active_work_area(&app)?;

    Ok(DesktopCapabilities {
        platform: std::env::consts::OS,
        work_area: monitor.map(|(work, _)| work),
        scale_factor: monitor.map(|(_, scale)| scale),
        drawer,
        material: material_kind(),
    })
}

#[tauri::command]
pub fn refresh_desktop_appearance(_app: AppHandle) -> Result<DesktopAppearance, String> {
    #[cfg(target_os = "macos")]
    let material = native::refresh_material(&_app)?;

    #[cfg(not(target_os = "macos"))]
    let material = material_kind();

    Ok(DesktopAppearance { material })
}

pub fn spawn_hover_watcher(app: AppHandle) {
    let _ = std::thread::spawn(move || {
        let mut last: Option<(i32, i32)> = None;
        let mut ignoring: Option<bool> = None;

        loop {
            std::thread::sleep(HOVER_POLL);

            let Some(cursor) = app.cursor_position().ok() else {
                continue;
            };
            let point = (cursor.x.round() as i32, cursor.y.round() as i32);
            let moved = last.is_some_and(|previous| previous != point);
            last = Some(point);

            let Some(form) = window_bounds(&app, NOTCH_LABEL) else {
                continue;
            };
            let work_area = active_work_area(&app).ok().flatten();
            let scale = work_area
                .map(|(_, scale)| scale)
                .unwrap_or_else(|| scale_of(&app));
            let hover_margin = scaled(HOVER_MARGIN, scale);

            let state = app.state::<Mutex<DesktopState>>();
            let (action, radius, corners) = match state.lock() {
                Ok(mut guard) => {
                    let view = guard.view();
                    let inside =
                        hover_contains(form, view, work_area, hover_margin, point.0, point.1);
                    let action = guard.pointer(inside, moved, Instant::now());
                    let view = guard.view();
                    (action, hit_test_radius(view), hit_test_corners(view))
                }
                Err(_) => (DrawerAction::Unchanged, RIBBON_RADIUS, RoundedCorners::Left),
            };

            let corner = contains(form, point.0, point.1)
                && outside_rounded_corners(form, radius * scale, corners, point.0, point.1);
            if ignoring != Some(corner)
                && let Some(window) = app.get_webview_window(NOTCH_LABEL)
            {
                let _ = window.set_ignore_cursor_events(corner);
                ignoring = Some(corner);
            }

            let _ = apply(&app, action);
        }
    });
}

pub fn blur_drawer(app: &AppHandle) {
    let state = app.state::<Mutex<DesktopState>>();
    let action = match state.lock() {
        Ok(mut guard) => guard.blur(),
        Err(_) => DrawerAction::Unchanged,
    };

    let _ = apply(app, action);
}

pub fn place_notch(app: &AppHandle) -> Result<(), String> {
    let Some((work, scale)) = active_work_area(app)? else {
        return Ok(());
    };
    let Some(window) = app.get_webview_window(NOTCH_LABEL) else {
        return Ok(());
    };

    let target = ribbon_rect(work, scale);
    window
        .set_size(PhysicalSize::new(target.width, target.height))
        .map_err(|error| error.to_string())?;
    window
        .set_position(PhysicalPosition::new(target.x, target.y))
        .map_err(|error| error.to_string())?;

    Ok(())
}

pub fn install_material(app: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        native::refresh_material(app).map(|_| ())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        Ok(())
    }
}

pub fn material_kind() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        native::material_kind()
    }

    #[cfg(not(target_os = "macos"))]
    {
        "solid"
    }
}

fn apply(app: &AppHandle, action: DrawerAction) -> Result<(), String> {
    let window = notch_window(app)?;
    match action {
        DrawerAction::Unchanged => Ok(()),
        DrawerAction::State { view, focus } => {
            #[cfg(target_os = "macos")]
            {
                native::apply_state(app, &window, view, focus)
            }

            #[cfg(not(target_os = "macos"))]
            {
                apply_fallback_state(app, &window, view, focus)
            }
        }
        DrawerAction::Transition { view, form, focus } => {
            let completion_after = move_form(app, &window, form, view, focus)?;
            if let Some(duration) = completion_after {
                schedule_transition_completion(app.clone(), view.generation, duration);
            }
            Ok(())
        }
    }
}

pub(super) fn emit_view(window: &WebviewWindow, view: DesktopView) -> Result<(), String> {
    window
        .emit(DRAWER_STATE_EVENT, view)
        .map_err(|error| error.to_string())
}

fn schedule_transition_completion(app: AppHandle, generation: u64, duration: Duration) {
    let finish = move || finish_transition(&app, generation);
    if duration.is_zero() {
        finish();
    } else {
        let _ = std::thread::spawn(move || {
            std::thread::sleep(duration);
            finish();
        });
    }
}

pub(super) fn finish_transition(app: &AppHandle, generation: u64) {
    let view = app
        .state::<Mutex<DesktopState>>()
        .lock()
        .ok()
        .and_then(|mut state| state.complete_transition(generation));

    if let Some(view) = view
        && let Some(window) = app.get_webview_window(NOTCH_LABEL)
    {
        let _ = emit_view(&window, view);
    }
}

#[cfg(not(target_os = "macos"))]
fn apply_fallback_state(
    app: &AppHandle,
    window: &WebviewWindow,
    view: DesktopView,
    focus: bool,
) -> Result<(), String> {
    let state = app.state::<Mutex<DesktopState>>();
    let state = state
        .lock()
        .map_err(|error| format!("Falha de sincronização interna: {error}"))?;
    if state.view().generation != view.generation {
        return Ok(());
    }

    if focus {
        window.set_focus().map_err(|error| error.to_string())?;
    }
    emit_view(window, view)
}

fn move_form(
    app: &AppHandle,
    window: &WebviewWindow,
    form: FormState,
    view: DesktopView,
    focus: bool,
) -> Result<Option<Duration>, String> {
    let Some((work, scale)) = active_work_area(app)? else {
        #[cfg(target_os = "macos")]
        {
            native::apply_state(app, window, view, focus)?;
            return Ok(Some(Duration::ZERO));
        }

        #[cfg(not(target_os = "macos"))]
        {
            apply_fallback_state(app, window, view, focus)?;
            return Ok(Some(Duration::ZERO));
        }
    };
    let target = match form {
        FormState::Open => open_rect(work, scale),
        FormState::Closed => ribbon_rect(work, scale),
    };

    #[cfg(target_os = "macos")]
    {
        native::animate_form(
            app,
            window,
            native::NativeTransition {
                target,
                scale,
                form,
                duration: transition_duration(form),
                view,
                focus,
            },
        )?;
        Ok(None)
    }

    #[cfg(not(target_os = "macos"))]
    {
        window
            .set_size(PhysicalSize::new(target.width, target.height))
            .map_err(|error| error.to_string())?;
        window
            .set_position(PhysicalPosition::new(target.x, target.y))
            .map_err(|error| error.to_string())?;
        apply_fallback_state(app, window, view, focus)?;
        Ok(Some(Duration::ZERO))
    }
}

#[cfg(target_os = "macos")]
fn transition_duration(form: FormState) -> Duration {
    match form {
        FormState::Open => Duration::from_millis(OPEN_MS),
        FormState::Closed => Duration::from_millis(CLOSE_MS),
    }
}

fn window_bounds(app: &AppHandle, label: &str) -> Option<Rect> {
    let window = app.get_webview_window(label)?;
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;

    Some(Rect {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

fn expand(rect: Rect, margin: u32) -> Rect {
    Rect {
        x: rect.x - margin as i32,
        y: rect.y - margin as i32,
        width: rect.width.saturating_add(margin.saturating_mul(2)),
        height: rect.height.saturating_add(margin.saturating_mul(2)),
    }
}

fn hover_contains(
    form: Rect,
    view: DesktopView,
    work_area: Option<(Rect, f64)>,
    margin: u32,
    x: i32,
    y: i32,
) -> bool {
    if contains(expand(form, margin), x, y) {
        return true;
    }

    if view.intent != DrawerIntent::Preview {
        return false;
    }

    let Some((work, scale)) = work_area else {
        return false;
    };
    let ribbon = ribbon_rect(work, scale);
    preview_hover_bridge(form, ribbon, work, margin).is_some_and(|bridge| contains(bridge, x, y))
}

fn preview_hover_bridge(form: Rect, ribbon: Rect, work: Rect, margin: u32) -> Option<Rect> {
    let form_right = form.x + form.width as i32;
    let work_right = work.x + work.width as i32;
    if form_right < work.x || form_right >= work_right {
        return None;
    }

    let top = (ribbon.y - margin as i32).max(work.y);
    let bottom = (ribbon.y + ribbon.height as i32 + margin as i32).min(work.y + work.height as i32);
    if bottom <= top {
        return None;
    }

    Some(Rect {
        x: form_right,
        y: top,
        width: (work_right - form_right) as u32,
        height: (bottom - top) as u32,
    })
}

fn contains(rect: Rect, x: i32, y: i32) -> bool {
    x >= rect.x && x < rect.x + rect.width as i32 && y >= rect.y && y < rect.y + rect.height as i32
}

fn outside_rounded_corners(
    rect: Rect,
    radius: f64,
    corners: RoundedCorners,
    x: i32,
    y: i32,
) -> bool {
    let radius = radius
        .min(f64::from(rect.width) / 2.0)
        .min(f64::from(rect.height) / 2.0);
    let px = f64::from(x);
    let py = f64::from(y);
    let left = f64::from(rect.x);
    let right = f64::from(rect.x + rect.width as i32);
    let top = f64::from(rect.y);
    let bottom = f64::from(rect.y + rect.height as i32);
    let center_y = if py < top + radius {
        top + radius
    } else if py >= bottom - radius {
        bottom - radius
    } else {
        return false;
    };
    let center_x = if px < left + radius {
        left + radius
    } else if corners == RoundedCorners::All && px >= right - radius {
        right - radius
    } else {
        return false;
    };

    let dx = px - center_x;
    let dy = py - center_y;
    dx * dx + dy * dy > radius * radius
}

fn hit_test_corners(view: DesktopView) -> RoundedCorners {
    match (view.intent, view.phase) {
        (DrawerIntent::Closed, DrawerPhase::Resting) => RoundedCorners::Left,
        _ => RoundedCorners::All,
    }
}

fn hit_test_radius(view: DesktopView) -> f64 {
    match (view.intent, view.phase) {
        (DrawerIntent::Closed, DrawerPhase::Resting) => RIBBON_RADIUS,
        (_, DrawerPhase::Opening | DrawerPhase::Closing)
        | (DrawerIntent::Preview | DrawerIntent::Pinned, DrawerPhase::Resting) => DRAWER_RADIUS,
    }
}

fn scale_of(app: &AppHandle) -> f64 {
    app.get_webview_window(NOTCH_LABEL)
        .and_then(|window| window.scale_factor().ok())
        .unwrap_or(1.0)
}

fn notch_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window(NOTCH_LABEL)
        .ok_or_else(|| "Janela da fita indisponível.".to_string())
}

fn active_work_area(app: &AppHandle) -> Result<Option<(Rect, f64)>, String> {
    let current = app
        .get_webview_window(NOTCH_LABEL)
        .and_then(|window| window.current_monitor().ok().flatten());
    let monitor = match current {
        Some(monitor) => Some(monitor),
        None => app.primary_monitor().map_err(|error| error.to_string())?,
    };
    let Some(monitor) = monitor else {
        return Ok(None);
    };
    let work = monitor.work_area();

    Ok(Some((
        Rect {
            x: work.position.x,
            y: work.position.y,
            width: work.size.width,
            height: work.size.height,
        },
        monitor.scale_factor(),
    )))
}

fn scaled(logical: u32, scale: f64) -> u32 {
    (f64::from(logical) * scale).round().max(1.0) as u32
}

pub fn ribbon_rect(work: Rect, scale: f64) -> Rect {
    let width = scaled(RIBBON_WIDTH, scale).min(work.width);
    let height = scaled(RIBBON_HEIGHT, scale).min(work.height);

    Rect {
        x: work.x + (work.width - width) as i32,
        y: work.y + ((work.height - height) / 2) as i32,
        width,
        height,
    }
}

pub fn open_rect(work: Rect, scale: f64) -> Rect {
    let ribbon = ribbon_rect(work, scale);
    let edge_margin = scaled(DRAWER_EDGE_MARGIN, scale).min(work.width.saturating_sub(1) / 2);
    let maximum_width = work
        .width
        .saturating_sub(edge_margin.saturating_mul(2))
        .max(1);
    let width = scaled(DRAWER_WIDTH, scale).min(maximum_width);
    let maximum_height = work
        .height
        .saturating_sub(scaled(DRAWER_HEIGHT_MARGIN, scale))
        .max(1);
    let height = scaled(DRAWER_MAX_HEIGHT, scale).min(maximum_height);
    let center = ribbon.y + (ribbon.height / 2) as i32;
    let max_y = work.y + (work.height - height) as i32;
    let y = (center - (height / 2) as i32).clamp(work.y, max_y.max(work.y));

    Rect {
        x: work.x + (work.width - edge_margin - width) as i32,
        y,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work_area() -> Rect {
        Rect {
            x: 0,
            y: 25,
            width: 1920,
            height: 1055,
        }
    }

    fn preview_state() -> (DesktopState, Instant) {
        let mut state = DesktopState::new();
        let start = Instant::now();
        assert_eq!(state.pointer(true, true, start), DrawerAction::Unchanged);
        assert!(matches!(
            state.pointer(true, false, start + HOVER_DWELL),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Preview,
                    phase: DrawerPhase::Opening,
                    generation: 2,
                },
                form: FormState::Open,
                focus: false,
            }
        ));
        (state, start)
    }

    #[test]
    fn hover_abre_previa_somente_apos_permanencia() {
        let mut state = DesktopState::new();
        let start = Instant::now();

        assert_eq!(state.pointer(true, true, start), DrawerAction::Unchanged);
        assert_eq!(
            state.pointer(true, false, start + HOVER_DWELL - Duration::from_millis(1)),
            DrawerAction::Unchanged
        );
        assert_eq!(state.view().intent, DrawerIntent::Closed);
        assert!(matches!(
            state.pointer(true, false, start + HOVER_DWELL),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Preview,
                    phase: DrawerPhase::Opening,
                    generation: 2,
                },
                form: FormState::Open,
                focus: false,
            }
        ));
        assert_eq!(state.view().intent, DrawerIntent::Preview);
    }

    #[test]
    fn ponteiro_parado_nao_inicia_permanencia() {
        let mut state = DesktopState::new();
        let start = Instant::now();

        assert_eq!(state.pointer(true, false, start), DrawerAction::Unchanged);
        assert_eq!(
            state.pointer(
                true,
                false,
                start + HOVER_DWELL + Duration::from_millis(200)
            ),
            DrawerAction::Unchanged
        );
        assert_eq!(state.view().intent, DrawerIntent::Closed);

        let moved_at = start + Duration::from_millis(300);
        assert_eq!(state.pointer(true, true, moved_at), DrawerAction::Unchanged);
        assert!(matches!(
            state.pointer(true, false, moved_at + HOVER_DWELL),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Preview,
                    phase: DrawerPhase::Opening,
                    generation: 2,
                },
                form: FormState::Open,
                focus: false,
            }
        ));
    }

    #[test]
    fn saida_da_previa_respeita_tolerancia_e_reentrada_cancela() {
        let (mut state, start) = preview_state();

        let left = start + Duration::from_millis(400);
        assert_eq!(state.pointer(false, true, left), DrawerAction::Unchanged);
        assert!(matches!(
            state.pointer(false, false, left + HOVER_EXIT_TOLERANCE),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Closed,
                    phase: DrawerPhase::Closing,
                    generation: 3,
                },
                form: FormState::Closed,
                focus: false,
            }
        ));
        assert_eq!(state.view().intent, DrawerIntent::Closed);

        let (mut state, start) = preview_state();
        let left = start + Duration::from_millis(200);
        assert_eq!(state.pointer(false, true, left), DrawerAction::Unchanged);
        assert_eq!(
            state.pointer(true, true, left + Duration::from_millis(60)),
            DrawerAction::Unchanged
        );
        assert_eq!(state.view().intent, DrawerIntent::Preview);
    }

    #[test]
    fn clique_fixa_a_previa_e_reabre_com_clique_na_fita() {
        let (mut state, _) = preview_state();

        assert!(matches!(
            state.toggle(),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Pinned,
                    phase: DrawerPhase::Opening,
                    generation: 3,
                },
                form: FormState::Open,
                focus: true,
            }
        ));
        assert_eq!(state.view().intent, DrawerIntent::Pinned);

        assert!(matches!(
            state.toggle(),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Closed,
                    phase: DrawerPhase::Closing,
                    generation: 4,
                },
                form: FormState::Closed,
                focus: false,
            }
        ));
        assert_eq!(state.view().intent, DrawerIntent::Closed);
        assert_eq!(
            state.pointer(true, true, Instant::now()),
            DrawerAction::Unchanged
        );
        assert_eq!(state.view().intent, DrawerIntent::Closed);
    }

    #[test]
    fn perda_de_foco_recolhe_previa_mas_preserva_fixada() {
        let (mut state, _) = preview_state();

        assert!(matches!(
            state.blur(),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Closed,
                    phase: DrawerPhase::Closing,
                    generation: 3,
                },
                form: FormState::Closed,
                focus: false,
            }
        ));
        assert_eq!(state.view().intent, DrawerIntent::Closed);

        let (mut state, _) = preview_state();
        let _ = state.toggle();
        assert_eq!(state.blur(), DrawerAction::Unchanged);
        assert_eq!(state.view().intent, DrawerIntent::Pinned);
    }

    #[test]
    fn recolher_fecha_previa_e_fixada() {
        let (mut state, _) = preview_state();
        assert!(matches!(
            state.collapse(),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Closed,
                    phase: DrawerPhase::Closing,
                    generation: 3,
                },
                form: FormState::Closed,
                focus: false,
            }
        ));

        let (mut state, _) = preview_state();
        let _ = state.toggle();
        assert!(matches!(
            state.collapse(),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Closed,
                    phase: DrawerPhase::Closing,
                    generation: 4,
                },
                form: FormState::Closed,
                focus: false,
            }
        ));
        assert_eq!(state.view().intent, DrawerIntent::Closed);
    }

    #[test]
    fn recolher_repetido_nao_gera_nova_geracao() {
        let mut state = DesktopState::new();
        assert_eq!(state.collapse(), DrawerAction::Unchanged);
        assert_eq!(state.view().generation, 1);
    }

    #[test]
    fn selecao_ou_captura_de_ponteiro_suspendem_o_recolhimento_automatico() {
        let (mut state, start) = preview_state();
        let left = start + Duration::from_millis(400);

        state.set_interaction_guards(InteractionGuards {
            selection: true,
            pointer_capture: false,
        });
        assert_eq!(state.pointer(false, true, left), DrawerAction::Unchanged);
        assert_eq!(
            state.pointer(false, false, left + HOVER_EXIT_TOLERANCE),
            DrawerAction::Unchanged
        );

        state.set_interaction_guards(InteractionGuards {
            selection: false,
            pointer_capture: true,
        });
        assert_eq!(
            state.pointer(false, false, left + HOVER_EXIT_TOLERANCE),
            DrawerAction::Unchanged
        );

        state.set_interaction_guards(InteractionGuards::default());
        assert_eq!(
            state.pointer(false, true, left + HOVER_EXIT_TOLERANCE),
            DrawerAction::Unchanged
        );
        assert!(matches!(
            state.pointer(
                false,
                false,
                left + HOVER_EXIT_TOLERANCE + HOVER_EXIT_TOLERANCE
            ),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Closed,
                    phase: DrawerPhase::Closing,
                    generation: 3,
                },
                form: FormState::Closed,
                focus: false,
            }
        ));
    }

    #[test]
    fn conclusao_obsoleta_nao_altera_a_transicao_atual() {
        let (mut state, _) = preview_state();
        let _ = state.collapse();

        assert_eq!(state.complete_transition(2), None);
        assert_eq!(state.view().phase, DrawerPhase::Closing);
        assert_eq!(
            state.complete_transition(3),
            Some(DesktopView {
                intent: DrawerIntent::Closed,
                phase: DrawerPhase::Resting,
                generation: 3,
            })
        );
    }

    #[test]
    fn fixar_durante_abertura_preserva_a_fase_e_invalida_a_conclusao_da_previa() {
        let (mut state, _) = preview_state();

        assert!(matches!(
            state.toggle(),
            DrawerAction::Transition {
                view: DesktopView {
                    intent: DrawerIntent::Pinned,
                    phase: DrawerPhase::Opening,
                    generation: 3,
                },
                form: FormState::Open,
                focus: true,
            }
        ));
        assert_eq!(state.complete_transition(2), None);
        assert_eq!(state.view().intent, DrawerIntent::Pinned);
        assert_eq!(state.view().phase, DrawerPhase::Opening);
        assert_eq!(
            state.complete_transition(3),
            Some(DesktopView {
                intent: DrawerIntent::Pinned,
                phase: DrawerPhase::Resting,
                generation: 3,
            })
        );
    }

    #[test]
    fn fita_gruda_na_borda_direita_e_centraliza() {
        let rect = ribbon_rect(work_area(), 1.0);

        assert_eq!(rect.width, RIBBON_WIDTH);
        assert_eq!(rect.height, RIBBON_HEIGHT);
        assert_eq!(rect.x + rect.width as i32, 1920);
        assert_eq!(rect.y + rect.height as i32 / 2, 25 + 1055 / 2);
    }

    #[test]
    fn forma_aberta_flutua_a_24_px_da_borda_e_centraliza_na_fita() {
        let rect = open_rect(work_area(), 1.0);

        assert_eq!(rect.width, DRAWER_WIDTH);
        assert_eq!(rect.height, DRAWER_MAX_HEIGHT);
        assert_eq!(rect.x + rect.width as i32, 1920 - DRAWER_EDGE_MARGIN as i32);
        assert_eq!(
            rect.y + rect.height as i32 / 2,
            ribbon_rect(work_area(), 1.0).y + RIBBON_HEIGHT as i32 / 2
        );
    }

    #[test]
    fn forma_aberta_respeita_area_util_pequena_e_reduzida() {
        let work = Rect {
            x: -1920,
            y: 0,
            width: 1280,
            height: 720,
        };
        let rect = open_rect(work, 1.0);

        assert_eq!(
            rect.x,
            -1920 + 1280 - DRAWER_EDGE_MARGIN as i32 - DRAWER_WIDTH as i32
        );
        assert_eq!(rect.x + rect.width as i32, -640 - DRAWER_EDGE_MARGIN as i32);
        assert_eq!(rect.height, DRAWER_MAX_HEIGHT);
        assert_eq!(rect.y, 60);
    }

    #[test]
    fn forma_aberta_deixa_margem_em_monitor_mais_estreito() {
        let work = Rect {
            x: -900,
            y: 0,
            width: 900,
            height: 600,
        };
        let rect = open_rect(work, 1.0);

        assert_eq!(rect.width, 852);
        assert_eq!(rect.height, 576);
        assert_eq!(rect.x, -876);
        assert_eq!(rect.y, 12);
    }

    #[test]
    fn margem_direita_escala_com_o_monitor_e_nao_desloca_a_origem_negativa() {
        let work = Rect {
            x: -2560,
            y: 80,
            width: 2880,
            height: 1800,
        };
        let rect = open_rect(work, 2.0);

        assert_eq!(rect.width, DRAWER_WIDTH * 2);
        assert_eq!(rect.height, DRAWER_MAX_HEIGHT * 2);
        assert_eq!(
            rect.x + rect.width as i32,
            320 - (DRAWER_EDGE_MARGIN * 2) as i32
        );
        assert!(rect.x >= work.x);
        assert!(rect.y >= work.y);
    }

    #[test]
    fn area_util_minima_preserva_a_forma_dentro_da_origem_negativa() {
        let work = Rect {
            x: -20,
            y: -10,
            width: 10,
            height: 10,
        };
        let rect = open_rect(work, 2.0);

        assert_eq!(rect.width, 2);
        assert_eq!(rect.height, 1);
        assert_eq!(rect.x, work.x + 4);
        assert!(rect.y >= work.y);
        assert!(rect.y + rect.height as i32 <= work.y + work.height as i32);
    }

    #[test]
    fn fita_acompanha_escala_e_cabe_na_area_util() {
        let rect = ribbon_rect(work_area(), 2.0);

        assert_eq!(rect.width, RIBBON_WIDTH * 2);
        assert_eq!(rect.height, RIBBON_HEIGHT * 2);
        assert_eq!(rect.x + rect.width as i32, 1920);

        let tiny = Rect {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        let clamped = ribbon_rect(tiny, 2.0);
        assert_eq!(clamped.width, 10);
        assert_eq!(clamped.height, 10);
        assert_eq!(clamped.x, 0);
        assert_eq!(clamped.y, 0);
    }

    #[test]
    fn forma_fechada_ignora_apenas_cantos_esquerdos_arredondados() {
        let rect = Rect {
            x: 100,
            y: 200,
            width: 120,
            height: 80,
        };

        assert!(outside_rounded_corners(
            rect,
            20.0,
            RoundedCorners::Left,
            100,
            200
        ));
        assert!(outside_rounded_corners(
            rect,
            20.0,
            RoundedCorners::Left,
            100,
            279
        ));
        assert!(!outside_rounded_corners(
            rect,
            20.0,
            RoundedCorners::Left,
            120,
            200
        ));
        assert!(!outside_rounded_corners(
            rect,
            20.0,
            RoundedCorners::Left,
            219,
            200
        ));
    }

    #[test]
    fn forma_aberta_ignora_os_quatro_cantos_arredondados() {
        let rect = Rect {
            x: 100,
            y: 200,
            width: 120,
            height: 80,
        };

        assert!(outside_rounded_corners(
            rect,
            20.0,
            RoundedCorners::All,
            100,
            200
        ));
        assert!(outside_rounded_corners(
            rect,
            20.0,
            RoundedCorners::All,
            219,
            200
        ));
        assert!(outside_rounded_corners(
            rect,
            20.0,
            RoundedCorners::All,
            219,
            279
        ));
    }

    #[test]
    fn morph_usa_o_maior_raio_para_nao_capturar_cantos_transparentes() {
        assert_eq!(
            hit_test_radius(DesktopView {
                intent: DrawerIntent::Closed,
                phase: DrawerPhase::Resting,
                generation: 1,
            }),
            RIBBON_RADIUS
        );
        assert_eq!(
            hit_test_radius(DesktopView {
                intent: DrawerIntent::Closed,
                phase: DrawerPhase::Closing,
                generation: 2,
            }),
            DRAWER_RADIUS
        );
        assert_eq!(
            hit_test_radius(DesktopView {
                intent: DrawerIntent::Preview,
                phase: DrawerPhase::Opening,
                generation: 2,
            }),
            DRAWER_RADIUS
        );
        assert_eq!(
            hit_test_corners(DesktopView {
                intent: DrawerIntent::Closed,
                phase: DrawerPhase::Resting,
                generation: 1,
            }),
            RoundedCorners::Left
        );
        assert_eq!(
            hit_test_corners(DesktopView {
                intent: DrawerIntent::Closed,
                phase: DrawerPhase::Closing,
                generation: 2,
            }),
            RoundedCorners::All
        );
    }

    #[test]
    fn corredor_da_previa_cobre_so_a_lacuna_da_fita_na_escala_atual() {
        let work = work_area();
        let scale = 2.0;
        let form = open_rect(work, scale);
        let ribbon = ribbon_rect(work, scale);
        let bridge = preview_hover_bridge(form, ribbon, work, scaled(HOVER_MARGIN, scale))
            .expect("a gaveta aberta deixa uma lacuna de hover");

        assert_eq!(bridge.x, form.x + form.width as i32);
        assert_eq!(bridge.x + bridge.width as i32, work.x + work.width as i32);
        assert!(contains(
            bridge,
            work.x + work.width as i32 - 1,
            ribbon.y + ribbon.height as i32 / 2
        ));
        assert!(!contains(bridge, work.x + work.width as i32 - 1, work.y));
    }

    #[test]
    fn ponteiro_parado_na_lacuna_nao_recolhe_a_previa() {
        let (mut state, start) = preview_state();
        let work = work_area();
        let scale = 2.0;
        let form = open_rect(work, scale);
        let point = (
            work.x + work.width as i32 - 1,
            ribbon_rect(work, scale).y + 20,
        );

        assert!(hover_contains(
            form,
            state.view(),
            Some((work, scale)),
            scaled(HOVER_MARGIN, scale),
            point.0,
            point.1
        ));
        assert_eq!(
            state.pointer(
                true,
                false,
                start + HOVER_EXIT_TOLERANCE + Duration::from_millis(60)
            ),
            DrawerAction::Unchanged
        );
        assert_eq!(state.view().intent, DrawerIntent::Preview);
    }
}
