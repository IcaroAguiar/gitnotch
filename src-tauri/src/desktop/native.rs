#![cfg(target_os = "macos")]

use std::sync::Mutex;
use std::time::Duration;

use block2::RcBlock;
use core::ptr::NonNull;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::runtime::AnyClass;
use objc2_app_kit::{
    NSAnimatablePropertyContainer, NSAnimationContext, NSAutoresizingMaskOptions,
    NSGlassEffectView, NSGlassEffectViewStyle, NSWindow, NSWorkspace,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::CAMediaTimingFunction;
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

use super::{DesktopView, DrawerIntent, DrawerPhase, FormState, Rect};

const RADIUS_CLOSED: f64 = 8.0;
const RADIUS_OPEN: f64 = 20.0;

static MATERIAL: Mutex<&'static str> = Mutex::new("solid");

pub struct NativeTransition {
    pub target: Rect,
    pub scale: f64,
    pub form: FormState,
    pub duration: Duration,
    pub view: DesktopView,
    pub focus: bool,
}

pub fn material_kind() -> &'static str {
    MATERIAL.lock().map(|kind| *kind).unwrap_or("solid")
}

pub fn refresh_material(app: &AppHandle) -> Result<&'static str, String> {
    let app = app.clone();
    let main_app = app.clone();
    let window = app
        .get_webview_window(super::NOTCH_LABEL)
        .ok_or_else(|| "Janela da fita indisponível.".to_string())?;

    let kind = run_on_main(&app, move || -> Result<&'static str, String> {
        let mtm = MainThreadMarker::new().ok_or_else(|| "main thread ausente".to_string())?;
        let ns = ns_window(&window)?;

        if AnyClass::get(c"NSGlassEffectView").is_none()
            || NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceTransparency()
        {
            clear_material(&ns);
            return Ok("solid");
        }

        let state = main_app.state::<Mutex<super::DesktopState>>();
        let state = state
            .lock()
            .map_err(|error| format!("Falha de sincronização interna: {error}"))?;
        let radius = material_radius(state.view());
        let result = install_material(&ns, mtm, radius);
        drop(state);
        result?;
        Ok("glass")
    })?;

    if let Ok(mut material) = MATERIAL.lock() {
        *material = kind;
    }
    Ok(kind)
}

fn install_material(ns: &NSWindow, mtm: MainThreadMarker, radius: f64) -> Result<(), String> {
    if glass_view(ns).is_some() {
        return Ok(());
    }

    let content = ns
        .contentView()
        .ok_or_else(|| "contentView indisponível".to_string())?;
    let glass = NSGlassEffectView::new(mtm);
    glass.setStyle(NSGlassEffectViewStyle::Regular);
    glass.setFrame(content.bounds());
    glass.setCornerRadius(radius);
    glass.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    content.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    glass.setContentView(Some(&content));
    ns.setContentView(Some(&glass));
    Ok(())
}

fn clear_material(ns: &NSWindow) {
    if let Some(glass) = glass_view(ns)
        && let Some(content) = glass.contentView()
    {
        ns.setContentView(Some(&content));
    }
}

fn glass_view(window: &NSWindow) -> Option<Retained<NSGlassEffectView>> {
    window.contentView()?.downcast::<NSGlassEffectView>().ok()
}

fn material_radius(view: DesktopView) -> f64 {
    match (view.intent, view.phase) {
        (DrawerIntent::Closed, DrawerPhase::Resting) => RADIUS_CLOSED,
        _ => RADIUS_OPEN,
    }
}

pub fn apply_state(
    app: &AppHandle,
    window: &WebviewWindow,
    view: DesktopView,
    focus: bool,
) -> Result<(), String> {
    let app = app.clone();
    let main_app = app.clone();
    let window = window.clone();

    run_on_main(&app, move || {
        let state = main_app.state::<Mutex<super::DesktopState>>();
        let state = state
            .lock()
            .map_err(|error| format!("Falha de sincronização interna: {error}"))?;
        if state.view().generation != view.generation {
            return Ok(());
        }

        if focus {
            window.set_focus().map_err(|error| error.to_string())?;
        }
        let result = super::emit_view(&window, view);
        drop(state);
        result
    })
}

pub fn animate_form(
    app: &AppHandle,
    window: &WebviewWindow,
    transition: NativeTransition,
) -> Result<(), String> {
    let NativeTransition {
        target,
        scale,
        form,
        duration: requested_duration,
        view,
        focus,
    } = transition;
    let app = app.clone();
    let main_app = app.clone();
    let completion_app = app.clone();
    let window = window.clone();

    run_on_main(&app, move || -> Result<(), String> {
        let _mtm = MainThreadMarker::new().ok_or_else(|| "main thread ausente".to_string())?;
        let ns = ns_window(&window)?;

        let reduced = NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion();
        let actual_duration = if reduced {
            Duration::ZERO
        } else {
            requested_duration
        };
        let current_position = window.outer_position().map_err(|error| error.to_string())?;
        let frame = target_frame_from_current(ns.frame(), current_position, target, scale);
        let radius = match form {
            FormState::Open => RADIUS_OPEN,
            FormState::Closed => RADIUS_CLOSED,
        };
        let timing = match form {
            FormState::Open => CAMediaTimingFunction::functionWithControlPoints(0.2, 0.8, 0.2, 1.0),
            FormState::Closed => {
                CAMediaTimingFunction::functionWithControlPoints(0.4, 0.0, 1.0, 1.0)
            }
        };
        let changes = RcBlock::new(move |context: NonNull<NSAnimationContext>| {
            // AppKit fornece este contexto apenas durante o grupo de animação.
            let context = unsafe { context.as_ref() };
            context.setDuration(actual_duration.as_secs_f64());
            context.setTimingFunction(Some(&timing));
            ns.animator().setFrame_display(frame, true);
            if let Some(glass) = glass_view(&ns) {
                glass.animator().setCornerRadius(radius);
            }
        });
        let completion = RcBlock::new(move || {
            let app = completion_app.clone();
            let _ = completion_app.run_on_main_thread(move || {
                super::finish_transition(&app, view.generation);
            });
        });

        let state = main_app.state::<Mutex<super::DesktopState>>();
        let state = state
            .lock()
            .map_err(|error| format!("Falha de sincronização interna: {error}"))?;
        if state.view().generation != view.generation {
            return Ok(());
        }
        if focus {
            window.set_focus().map_err(|error| error.to_string())?;
        }
        super::emit_view(&window, view)?;
        NSAnimationContext::runAnimationGroup_completionHandler(&changes, Some(&completion));
        drop(state);

        Ok(())
    })
}

fn target_frame_from_current(
    current: NSRect,
    current_position: PhysicalPosition<i32>,
    target: Rect,
    scale: f64,
) -> NSRect {
    let target_width = f64::from(target.width) / scale;
    let target_height = f64::from(target.height) / scale;
    let delta_x = f64::from(target.x - current_position.x) / scale;
    let delta_y_from_top = f64::from(target.y - current_position.y) / scale;

    NSRect::new(
        NSPoint::new(
            current.origin.x + delta_x,
            current.origin.y - delta_y_from_top - (target_height - current.size.height),
        ),
        NSSize::new(target_width, target_height),
    )
}

fn ns_window(window: &WebviewWindow) -> Result<Retained<NSWindow>, String> {
    let raw = window.ns_window().map_err(|error| error.to_string())? as *mut NSWindow;
    unsafe { Retained::retain(raw) }.ok_or_else(|| "janela nativa indisponível".to_string())
}

fn run_on_main<T, F>(app: &AppHandle, task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    if MainThreadMarker::new().is_some() {
        return task();
    }

    let (sender, receiver) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = sender.send(task());
    })
    .map_err(|error| error.to_string())?;

    receiver
        .recv_timeout(Duration::from_millis(1500))
        .map_err(|_| "tempo esgotado na main thread".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversao_preserva_origem_negativa_e_escala_do_monitor_atual() {
        let current = NSRect::new(NSPoint::new(-960.0, 310.0), NSSize::new(8.0, 48.0));
        let frame = target_frame_from_current(
            current,
            PhysicalPosition::new(-1920, 800),
            Rect {
                x: -5120,
                y: 640,
                width: 1920,
                height: 1200,
            },
            2.0,
        );

        assert_eq!(frame.origin.x, -2560.0);
        assert_eq!(frame.origin.y, -162.0);
        assert_eq!(frame.size.width, 960.0);
        assert_eq!(frame.size.height, 600.0);
    }

    #[test]
    fn material_reinstalado_recebe_o_raio_da_forma_atual() {
        assert_eq!(
            material_radius(DesktopView {
                intent: DrawerIntent::Closed,
                phase: DrawerPhase::Resting,
                generation: 1,
            }),
            RADIUS_CLOSED
        );
        assert_eq!(
            material_radius(DesktopView {
                intent: DrawerIntent::Pinned,
                phase: DrawerPhase::Opening,
                generation: 2,
            }),
            RADIUS_OPEN
        );
    }
}
