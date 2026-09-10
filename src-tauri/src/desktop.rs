use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, State, WebviewWindow};

pub const NOTCH_LABEL: &str = "notch";
pub const DRAWER_LABEL: &str = "drawer";

const DRAWER_WIDTH: u32 = 600;
const DRAWER_MAX_HEIGHT: u32 = 800;
const DRAWER_MARGIN: u32 = 12;
const DRAWER_HEIGHT_MARGIN: u32 = 24;
const TOGGLE_AFTER_COLLAPSE: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DrawerState {
    Closed,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopView {
    pub drawer: DrawerState,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerAction {
    Show,
    Hide,
    Unchanged,
}

#[derive(Debug)]
pub struct DesktopState {
    drawer: DrawerState,
    generation: u64,
    last_collapse: Option<Instant>,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopState {
    pub fn new() -> Self {
        Self {
            drawer: DrawerState::Closed,
            generation: 1,
            last_collapse: None,
        }
    }

    pub fn view(&self) -> DesktopView {
        DesktopView {
            drawer: self.drawer,
            generation: self.generation,
        }
    }

    pub fn toggle(&mut self, now: Instant) -> DrawerAction {
        match self.drawer {
            DrawerState::Open => self.close(),
            DrawerState::Closed => {
                let collapsed_recently = self
                    .last_collapse
                    .is_some_and(|at| now.saturating_duration_since(at) < TOGGLE_AFTER_COLLAPSE);

                if collapsed_recently {
                    DrawerAction::Unchanged
                } else {
                    self.open()
                }
            }
        }
    }

    pub fn collapse(&mut self, now: Instant) -> DrawerAction {
        match self.drawer {
            DrawerState::Open => {
                self.last_collapse = Some(now);
                self.close()
            }
            DrawerState::Closed => DrawerAction::Unchanged,
        }
    }

    fn open(&mut self) -> DrawerAction {
        self.drawer = DrawerState::Open;
        self.generation += 1;
        DrawerAction::Show
    }

    fn close(&mut self) -> DrawerAction {
        self.drawer = DrawerState::Closed;
        self.generation += 1;
        DrawerAction::Hide
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopCapabilities {
    pub platform: &'static str,
    pub work_area: Option<Rect>,
    pub scale_factor: Option<f64>,
    pub drawer: DesktopView,
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
        let action = guard.toggle(Instant::now());
        (action, guard.view())
    };

    apply(&app, action)?;
    Ok(view)
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
    let monitor = primary_work_area(&app)?;

    Ok(DesktopCapabilities {
        platform: std::env::consts::OS,
        work_area: monitor.map(|(work, _)| work),
        scale_factor: monitor.map(|(_, scale)| scale),
        drawer,
    })
}

pub fn place_notch(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(NOTCH_LABEL) else {
        return Ok(());
    };
    let Some((work, _)) = primary_work_area(app)? else {
        return Ok(());
    };

    let size = window.outer_size().map_err(|error| error.to_string())?;
    let (x, y) = notch_position(work, (size.width, size.height));
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|error| error.to_string())
}

pub fn collapse_drawer(app: &AppHandle) {
    let state = app.state::<Mutex<DesktopState>>();
    let action = match state.lock() {
        Ok(mut guard) => guard.collapse(Instant::now()),
        Err(_) => DrawerAction::Unchanged,
    };

    let _ = apply(app, action);
}

fn apply(app: &AppHandle, action: DrawerAction) -> Result<(), String> {
    match action {
        DrawerAction::Unchanged => Ok(()),
        DrawerAction::Show => show_drawer(app),
        DrawerAction::Hide => hide_drawer(app),
    }
}

fn show_drawer(app: &AppHandle) -> Result<(), String> {
    let window = drawer_window(app)?;
    place_drawer(app, &window)?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

fn hide_drawer(app: &AppHandle) -> Result<(), String> {
    drawer_window(app)?
        .hide()
        .map_err(|error| error.to_string())
}

fn drawer_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window(DRAWER_LABEL)
        .ok_or_else(|| "Janela da gaveta indisponível.".to_string())
}

fn place_drawer(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let Some((work, scale)) = primary_work_area(app)? else {
        return Ok(());
    };

    let width = scaled(DRAWER_WIDTH, scale).min(work.width);
    let height = scaled(DRAWER_MAX_HEIGHT, scale).min(
        work.height
            .saturating_sub(scaled(DRAWER_HEIGHT_MARGIN, scale)),
    );
    window
        .set_size(PhysicalSize::new(width, height))
        .map_err(|error| error.to_string())?;

    let notch = notch_bounds(app, work)?;
    let (x, y) = drawer_position(work, notch, (width, height), scaled(DRAWER_MARGIN, scale));
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|error| error.to_string())
}

fn notch_bounds(app: &AppHandle, work: Rect) -> Result<Rect, String> {
    let Some(notch) = app.get_webview_window(NOTCH_LABEL) else {
        let (x, y) = notch_position(work, (0, 0));
        return Ok(Rect {
            x,
            y,
            width: 0,
            height: 0,
        });
    };

    let position = notch.outer_position().map_err(|error| error.to_string())?;
    let size = notch.outer_size().map_err(|error| error.to_string())?;

    Ok(Rect {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

fn primary_work_area(app: &AppHandle) -> Result<Option<(Rect, f64)>, String> {
    let Some(monitor) = app.primary_monitor().map_err(|error| error.to_string())? else {
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

pub fn notch_position(work: Rect, size: (u32, u32)) -> (i32, i32) {
    let x = i64::from(work.x) + i64::from(work.width) - i64::from(size.0);
    let y = i64::from(work.y) + (i64::from(work.height) - i64::from(size.1)) / 2;

    (
        x.clamp(i64::from(work.x), edge(work.x, work.width, size.0)) as i32,
        y.clamp(i64::from(work.y), edge(work.y, work.height, size.1)) as i32,
    )
}

pub fn drawer_position(work: Rect, notch: Rect, size: (u32, u32), margin: u32) -> (i32, i32) {
    let right = i64::from(notch.x) - i64::from(margin);
    let x = right - i64::from(size.0);
    let y = i64::from(notch.y) + (i64::from(notch.height) - i64::from(size.1)) / 2;

    (
        x.clamp(i64::from(work.x), edge(work.x, work.width, size.0)) as i32,
        y.clamp(i64::from(work.y), edge(work.y, work.height, size.1)) as i32,
    )
}

fn edge(start: i32, extent: u32, size: u32) -> i64 {
    (i64::from(start) + i64::from(extent) - i64::from(size)).max(i64::from(start))
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

    #[test]
    fn notch_encosta_na_borda_direita_e_centraliza() {
        assert_eq!(notch_position(work_area(), (28, 64)), (1892, 520));
    }

    #[test]
    fn notch_fica_no_limite_de_area_util_menor_que_a_aba() {
        let work = Rect {
            x: 0,
            y: 0,
            width: 20,
            height: 30,
        };

        assert_eq!(notch_position(work, (28, 64)), (0, 0));
    }

    #[test]
    fn notch_respeita_origem_negativa_de_monitor_secundario() {
        let work = Rect {
            x: -2560,
            y: 0,
            width: 2560,
            height: 1400,
        };

        assert_eq!(notch_position(work, (56, 128)), (-56, 636));
    }

    #[test]
    fn drawer_encosta_com_margem_e_centraliza_na_aba() {
        let notch = Rect {
            x: 1892,
            y: 520,
            width: 28,
            height: 64,
        };

        assert_eq!(
            drawer_position(work_area(), notch, (600, 800), 12),
            (1280, 152)
        );
    }

    #[test]
    fn drawer_fica_dentro_da_area_util_quando_a_aba_esta_no_topo_ou_na_base() {
        let no_topo = Rect {
            x: 1000,
            y: 25,
            width: 28,
            height: 64,
        };
        let na_base = Rect {
            x: 1000,
            y: 1016,
            width: 28,
            height: 64,
        };

        assert_eq!(
            drawer_position(work_area(), no_topo, (600, 800), 12),
            (388, 25)
        );
        assert_eq!(
            drawer_position(work_area(), na_base, (600, 800), 12),
            (388, 280)
        );
    }

    #[test]
    fn drawer_maior_que_area_util_ocupa_o_canto_superior_esquerdo() {
        let work = Rect {
            x: -800,
            y: 40,
            width: 500,
            height: 300,
        };
        let notch = Rect {
            x: -300,
            y: 140,
            width: 28,
            height: 64,
        };

        assert_eq!(drawer_position(work, notch, (600, 800), 12), (-800, 40));
    }

    #[test]
    fn toggle_abre_com_geracao_nova_e_fecha_em_seguida() {
        let mut state = DesktopState::new();
        let now = Instant::now();

        assert_eq!(state.view().drawer, DrawerState::Closed);
        assert_eq!(state.toggle(now), DrawerAction::Show);
        assert_eq!(state.view().drawer, DrawerState::Open);
        assert_eq!(state.view().generation, 2);
        assert_eq!(state.toggle(now), DrawerAction::Hide);
        assert_eq!(state.view().drawer, DrawerState::Closed);
        assert_eq!(state.view().generation, 3);
    }

    #[test]
    fn colapso_fecha_apenas_quando_aberta() {
        let mut state = DesktopState::new();
        let now = Instant::now();

        assert_eq!(state.collapse(now), DrawerAction::Unchanged);
        assert_eq!(state.toggle(now), DrawerAction::Show);
        assert_eq!(state.collapse(now), DrawerAction::Hide);
        assert_eq!(state.view().drawer, DrawerState::Closed);
    }

    #[test]
    fn clique_na_aba_logo_apos_perda_de_foco_nao_reabre_a_gaveta() {
        let mut state = DesktopState::new();
        let now = Instant::now();

        assert_eq!(state.toggle(now), DrawerAction::Show);
        assert_eq!(state.collapse(now), DrawerAction::Hide);
        assert_eq!(
            state.toggle(now + Duration::from_millis(100)),
            DrawerAction::Unchanged
        );
        assert_eq!(state.view().drawer, DrawerState::Closed);
        assert_eq!(
            state.toggle(now + Duration::from_millis(600)),
            DrawerAction::Show
        );
    }
}
