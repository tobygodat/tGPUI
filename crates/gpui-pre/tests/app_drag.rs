//! A drag handed to the platform as [`ExternalDragPayload::App`] keeps its typed value in the
//! app, comes back to whichever window of the app it enters, and shows its view as the drag icon.

use gpui::{
    AnyWindowHandle, App, AppContext as _, Context, DevicePixels, ExternalDragPayload,
    ExternalPaths, FileDropEvent, InputEvent as _, InteractiveElement as _, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, Render, StatefulInteractiveElement as _, Styled,
    TestAppContext, Window, div, point, px, size,
};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
};

#[derive(Clone, Debug, PartialEq)]
struct Value(u32);

struct Source {
    value: u32,
    released_outside: Rc<Cell<u32>>,
}

impl Render for Source {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let released_outside = self.released_outside.clone();
        div()
            .id("app-drag")
            .size_full()
            .on_drag(Value(self.value), |_, _, _, cx| cx.new(|_| Preview))
            .external_drag_payload(|_: &Value, _, _| Some(ExternalDragPayload::App))
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                if let Some(Value(value)) = cx.active_drag_value::<Value>() {
                    released_outside.set(released_outside.get() + value);
                }
            })
    }
}

struct Preview;

impl Render for Preview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(40.)).h(px(10.))
    }
}

struct Target(Rc<RefCell<Vec<Value>>>);

impl Render for Target {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let drops = self.0.clone();
        div()
            .size_full()
            .on_drop(move |value: &Value, _, _| drops.borrow_mut().push(value.clone()))
    }
}

/// Opens a source window, starts dragging `value` and moves the pointer out of the window, so
/// the platform takes the drag.
fn drag_out(cx: &mut TestAppContext, value: u32) -> (AnyWindowHandle, Rc<Cell<u32>>) {
    drag_out_then(cx, value, |_| {})
}

/// [`drag_out`], running `then` before the source draws again.
fn drag_out_then(
    cx: &mut TestAppContext,
    value: u32,
    then: impl FnOnce(&mut App),
) -> (AnyWindowHandle, Rc<Cell<u32>>) {
    let released_outside = Rc::new(Cell::new(0));
    let source: AnyWindowHandle = cx
        .add_window({
            let released_outside = released_outside.clone();
            move |_, _| Source {
                value,
                released_outside,
            }
        })
        .into();
    cx.set_start_external_drag_result(source, true);
    cx.update_window(source, |_, window, cx| {
        window.draw(cx).clear(cx);
        window.dispatch_event(
            MouseDownEvent {
                position: point(px(10.), px(10.)),
                button: MouseButton::Left,
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        for position in [point(px(20.), px(20.)), point(px(-1.), px(20.))] {
            window.dispatch_event(
                MouseMoveEvent {
                    position,
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
        }
        assert!(!cx.has_active_drag(), "the platform owns the drag");
        then(cx);
        window.draw(cx).clear(cx);
    })
    .unwrap();
    (source, released_outside)
}

#[gpui::test]
fn the_drag_icon_shows_when_another_window_takes_the_drag_before_the_source_draws(
    cx: &mut TestAppContext,
) {
    let destination: AnyWindowHandle = cx
        .add_window(|_, _| Target(Rc::new(RefCell::new(Vec::new()))))
        .into();
    cx.update_window(destination, |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    let (source, _) = drag_out_then(cx, 5, |cx| {
        cx.update_window(destination, |_, window, cx| {
            window.dispatch_event(
                FileDropEvent::AppDragEntered {
                    position: point(px(30.), px(30.)),
                }
                .to_platform_input(),
                cx,
            );
            assert!(cx.has_active_drag(), "the destination took the drag");
        })
        .unwrap();
    });
    assert_eq!(cx.external_drag_icons(source).len(), 1);
}

#[gpui::test]
fn app_drag_round_trips_typed_payload(cx: &mut TestAppContext) {
    let (source, released_outside) = drag_out(cx, 7);
    assert_eq!(cx.external_app_drags(source), 1);
    assert_eq!(
        cx.external_drag_icons(source),
        [(
            size(DevicePixels(80), DevicePixels(20)),
            point(px(20.), px(20.))
        )],
        "the preview is drawn once, at the window's scale, held where the pointer was when the drag began"
    );

    let drops = Rc::new(RefCell::new(Vec::new()));
    let destination: AnyWindowHandle = cx
        .add_window({
            let drops = drops.clone();
            move |_, _| Target(drops)
        })
        .into();
    let inside = point(px(30.), px(30.));
    cx.update_window(destination, |_, window, cx| {
        window.draw(cx).clear(cx);
        window.dispatch_event(
            FileDropEvent::AppDragEntered { position: inside }.to_platform_input(),
            cx,
        );
        assert!(cx.has_active_drag(), "the window takes the drag back");
        window.dispatch_event(FileDropEvent::Exited.to_platform_input(), cx);
        assert!(
            !cx.has_active_drag(),
            "leaving hands it to the platform again"
        );

        window.dispatch_event(
            FileDropEvent::AppDragEntered { position: inside }.to_platform_input(),
            cx,
        );
        window.draw(cx).clear(cx);
        window.dispatch_event(
            FileDropEvent::Submit { position: inside }.to_platform_input(),
            cx,
        );
        assert!(!cx.has_active_drag());
    })
    .unwrap();
    assert_eq!(drops.borrow().as_slice(), [Value(7)]);
    cx.update_window(source, |_, window, cx| {
        window.dispatch_event(FileDropEvent::Ended.to_platform_input(), cx);
    })
    .unwrap();
    assert_eq!(released_outside.get(), 0);
    cx.update_window(destination, |_, window, cx| {
        window.dispatch_event(
            FileDropEvent::AppDragEntered { position: inside }.to_platform_input(),
            cx,
        );
        assert!(!cx.has_active_drag(), "an ended drag cannot come back");
        window.dispatch_event(FileDropEvent::Exited.to_platform_input(), cx);
    })
    .unwrap();

    let (source, released_outside) = drag_out(cx, 8);
    cx.update_window(source, |_, window, cx| {
        window.dispatch_event(FileDropEvent::AppDragDroppedOutside.to_platform_input(), cx);
        assert!(!cx.has_active_drag());
        window.dispatch_event(FileDropEvent::Ended.to_platform_input(), cx);
    })
    .unwrap();
    assert_eq!(
        released_outside.get(),
        8,
        "a drop where no window is reaches the source, with its value, as a release outside it"
    );
    assert_eq!(drops.borrow().len(), 1);

    let (source, released_outside) = drag_out(cx, 9);
    cx.update_window(source, |_, window, cx| {
        window.dispatch_event(FileDropEvent::Ended.to_platform_input(), cx);
        assert!(!cx.has_active_drag());
    })
    .unwrap();
    cx.update_window(destination, |_, window, cx| {
        window.dispatch_event(
            FileDropEvent::Entered {
                position: inside,
                paths: ExternalPaths([PathBuf::from("/tmp/foreign")].into_iter().collect()),
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            FileDropEvent::Submit { position: inside }.to_platform_input(),
            cx,
        );
    })
    .unwrap();
    assert_eq!(
        released_outside.get(),
        0,
        "a cancelled drag changes nothing"
    );
    assert_eq!(
        drops.borrow().len(),
        1,
        "a foreign file drop is not the app's value"
    );
}
