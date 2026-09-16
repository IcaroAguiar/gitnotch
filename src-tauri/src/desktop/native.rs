#![cfg(target_os = "macos")]

use std::sync::Mutex;
use std::time::Duration;

use block2::RcBlock;
use core::ptr::NonNull;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::runtime::AnyClass;
use objc2_app_kit::{
    NSAnimatablePropertyContainer, NSAnimationContext, NSAppearance, NSAppearanceCustomization,
    NSAppearanceNameAqua, NSAutoresizingMaskOptions, NSGlassEffectView, NSGlassEffectViewStyle,
    NSView, NSWindow, NSWorkspace,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::CAMediaTimingFunction;
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

use super::{DesktopView, DrawerIntent, DrawerPhase, FormState, Rect};

const GLASS_RIGHT_OVERSCAN: f64 = super::DRAWER_RADIUS;

static MATERIAL: Mutex<&'static str> = Mutex::new("solid");

#[derive(Clone, Copy)]
struct MaterialFrames {
    glass: NSRect,
    host: NSRect,
    content: NSRect,
}

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

        let state = main_app.state::<Mutex<super::DesktopState>>();
        let state = state
            .lock()
            .map_err(|error| format!("Falha de sincronização interna: {error}"))?;
        let view = state.view();

        if AnyClass::get(c"NSGlassEffectView").is_none()
            || NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceTransparency()
        {
            clear_material(&ns)?;
            drop(state);
            return Ok("solid");
        }

        install_material(&ns, mtm, view)?;
        drop(state);
        Ok("glass")
    })?;

    if let Ok(mut material) = MATERIAL.lock() {
        *material = kind;
    }
    Ok(kind)
}

fn install_material(ns: &NSWindow, mtm: MainThreadMarker, view: DesktopView) -> Result<(), String> {
    if let Some(glass) = glass_view(ns) {
        set_material_appearance(&glass, view)?;
        if view.phase == DrawerPhase::Resting {
            reconcile_material_frames(ns, material_form(view))?;
        }
        return Ok(());
    }

    let content = ns
        .contentView()
        .ok_or_else(|| "contentView indisponível".to_string())?;
    let visible_bounds = content.bounds();
    let clip = NSView::initWithFrame(mtm.alloc(), visible_bounds);
    clip.setClipsToBounds(true);
    clip.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );

    let frames = material_frames(visible_bounds, material_form(view));
    let glass = NSGlassEffectView::new(mtm);
    glass.setStyle(NSGlassEffectViewStyle::Regular);
    set_material_appearance(&glass, view)?;
    glass.setFrame(frames.glass);
    glass.setCornerRadius(material_radius(view));
    glass.setAutoresizingMask(material_autoresizing_mask(view.phase));
    let host = NSView::initWithFrame(mtm.alloc(), frames.host);
    host.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    content.setFrame(frames.content);
    content.setAutoresizingMask(material_autoresizing_mask(view.phase));
    ns.setContentView(None);
    host.addSubview(&content);
    glass.setContentView(Some(&host));
    clip.addSubview(&glass);
    ns.setContentView(Some(&clip));
    Ok(())
}

fn clear_material(ns: &NSWindow) -> Result<(), String> {
    let Some(views) = material_views(ns)? else {
        return Ok(());
    };

    views.content.removeFromSuperview();
    views.glass.setContentView(None);
    views.content.setFrame(views.clip.bounds());
    views.content.setAutoresizingMask(
        NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable,
    );
    ns.setContentView(Some(&views.content));
    Ok(())
}

#[derive(Clone)]
struct MaterialViews {
    clip: Retained<NSView>,
    glass: Retained<NSGlassEffectView>,
    host: Retained<NSView>,
    content: Retained<NSView>,
}

fn material_views(ns: &NSWindow) -> Result<Option<MaterialViews>, String> {
    let Some(clip) = ns.contentView() else {
        return Ok(None);
    };
    let Some(glass) = clip
        .subviews()
        .firstObject()
        .and_then(|view| view.downcast::<NSGlassEffectView>().ok())
    else {
        return Ok(None);
    };
    let host = glass
        .contentView()
        .ok_or_else(|| "host do material indisponível".to_string())?;
    let content = host
        .subviews()
        .firstObject()
        .ok_or_else(|| "conteúdo do material indisponível".to_string())?;

    Ok(Some(MaterialViews {
        clip,
        glass,
        host,
        content,
    }))
}

fn glass_view(window: &NSWindow) -> Option<Retained<NSGlassEffectView>> {
    window
        .contentView()?
        .subviews()
        .firstObject()?
        .downcast::<NSGlassEffectView>()
        .ok()
}

fn material_frames(visible_bounds: NSRect, form: FormState) -> MaterialFrames {
    let glass = NSRect::new(
        visible_bounds.origin,
        NSSize::new(
            visible_bounds.size.width + material_overscan(form),
            visible_bounds.size.height,
        ),
    );
    MaterialFrames {
        glass,
        host: NSRect::new(NSPoint::new(0.0, 0.0), glass.size),
        content: visible_bounds,
    }
}

fn material_overscan(form: FormState) -> f64 {
    match form {
        FormState::Closed => GLASS_RIGHT_OVERSCAN,
        FormState::Open => 0.0,
    }
}

fn reconcile_material_frames(ns: &NSWindow, form: FormState) -> Result<(), String> {
    let Some(views) = material_views(ns)? else {
        return Ok(());
    };
    let frames = material_frames(views.clip.bounds(), form);
    views.glass.setFrame(frames.glass);
    views.host.setFrame(frames.host);
    views.content.setFrame(frames.content);
    views
        .glass
        .setAutoresizingMask(material_autoresizing_mask(DrawerPhase::Resting));
    views
        .content
        .setAutoresizingMask(material_autoresizing_mask(DrawerPhase::Resting));
    Ok(())
}

fn prepare_material_animation(views: &MaterialViews) {
    views
        .glass
        .setAutoresizingMask(material_autoresizing_mask(DrawerPhase::Opening));
    views
        .content
        .setAutoresizingMask(material_autoresizing_mask(DrawerPhase::Opening));
}

fn material_autoresizing_mask(phase: DrawerPhase) -> NSAutoresizingMaskOptions {
    let mask = NSAutoresizingMaskOptions::ViewHeightSizable;
    if phase == DrawerPhase::Resting {
        mask | NSAutoresizingMaskOptions::ViewWidthSizable
    } else {
        mask
    }
}

fn target_visible_bounds(visible_bounds: NSRect, target: Rect, scale: f64) -> NSRect {
    NSRect::new(
        visible_bounds.origin,
        NSSize::new(
            f64::from(target.width) / scale,
            f64::from(target.height) / scale,
        ),
    )
}

fn material_radius(view: DesktopView) -> f64 {
    match material_form(view) {
        FormState::Closed => super::RIBBON_RADIUS,
        FormState::Open => super::DRAWER_RADIUS,
    }
}

fn material_form(view: DesktopView) -> FormState {
    match (view.intent, view.phase) {
        (DrawerIntent::Closed, DrawerPhase::Resting) => FormState::Closed,
        _ => FormState::Open,
    }
}

fn set_material_appearance(glass: &NSGlassEffectView, view: DesktopView) -> Result<(), String> {
    let current = glass.appearance();
    match material_form(view) {
        FormState::Closed => {
            if current.is_some() {
                glass.setAppearance(None);
            }
        }
        FormState::Open => {
            let aqua = NSAppearance::appearanceNamed(unsafe { NSAppearanceNameAqua })
                .ok_or_else(|| "aparência Aqua indisponível".to_string())?;
            let aqua_name = aqua.name().to_string();
            let current_name = current.map(|appearance| appearance.name().to_string());
            if current_name.as_deref() != Some(aqua_name.as_str()) {
                glass.setAppearance(Some(&aqua));
            }
        }
    }
    Ok(())
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
        let ns = ns_window(&window)?;
        let state = main_app.state::<Mutex<super::DesktopState>>();
        let state = state
            .lock()
            .map_err(|error| format!("Falha de sincronização interna: {error}"))?;
        if state.view().generation != view.generation {
            return Ok(());
        }

        if let Some(glass) = glass_view(&ns) {
            set_material_appearance(&glass, view)?;
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
        let material = material_views(&ns)?;
        let target_material_frames = material.as_ref().map(|views| {
            material_frames(
                target_visible_bounds(views.clip.bounds(), target, scale),
                form,
            )
        });
        let radius = match form {
            FormState::Open => super::DRAWER_RADIUS,
            FormState::Closed => super::RIBBON_RADIUS,
        };
        let timing = match form {
            FormState::Open => {
                CAMediaTimingFunction::functionWithControlPoints(0.16, 1.0, 0.3, 1.0)
            }
            FormState::Closed => {
                CAMediaTimingFunction::functionWithControlPoints(0.32, 0.72, 0.0, 1.0)
            }
        };
        let animation_ns = ns.clone();
        let animation_material = material.clone();
        let changes = RcBlock::new(move |context: NonNull<NSAnimationContext>| {
            // AppKit fornece este contexto apenas durante o grupo de animação.
            let context = unsafe { context.as_ref() };
            context.setDuration(actual_duration.as_secs_f64());
            context.setTimingFunction(Some(&timing));
            animation_ns.animator().setFrame_display(frame, true);
            if let Some(glass) = glass_view(&animation_ns) {
                glass.animator().setCornerRadius(radius);
            }
            if let (Some(views), Some(frames)) = (&animation_material, target_material_frames) {
                views.glass.animator().setFrame(frames.glass);
                views.content.animator().setFrame(frames.content);
            }
        });
        let completion = RcBlock::new(move || {
            let app = completion_app.clone();
            let _ = completion_app.run_on_main_thread(move || {
                super::finish_transition(&app, view.generation);
                let _ = refresh_material(&app);
            });
        });

        let state = main_app.state::<Mutex<super::DesktopState>>();
        let state = state
            .lock()
            .map_err(|error| format!("Falha de sincronização interna: {error}"))?;
        if state.view().generation != view.generation {
            return Ok(());
        }

        if let Some(glass) = glass_view(&ns) {
            set_material_appearance(&glass, view)?;
        }
        if let Some(views) = material.as_ref() {
            prepare_material_animation(views);
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
            super::super::RIBBON_RADIUS
        );
        assert_eq!(
            material_radius(DesktopView {
                intent: DrawerIntent::Pinned,
                phase: DrawerPhase::Opening,
                generation: 2,
            }),
            super::super::DRAWER_RADIUS
        );
    }

    #[test]
    fn vidro_anima_o_excedente_sem_alargar_o_conteudo_visivel() {
        let visible = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(480.0, 300.0));
        let closed = material_frames(visible, FormState::Closed);
        let open = material_frames(visible, FormState::Open);

        assert_eq!(closed.glass.size.width, 500.0);
        assert_eq!(closed.host.size.width, 500.0);
        assert_eq!(closed.content.size.width, 480.0);
        assert_eq!(open.glass.size.width, 480.0);
        assert_eq!(open.host.size.width, 480.0);
        assert_eq!(open.content.size.width, 480.0);
    }
}
