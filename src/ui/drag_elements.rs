use dioxus::prelude::*;
use dioxus_html::input_data::MouseButton;

use crate::ui::AppDocument;

#[derive(Clone)]
pub struct Dragging(pub Signal<bool>);

/// Css style of element to move it.
#[derive(Clone)]
pub struct PositionString(pub Memo<String>);

/// Current computed position to move drag element to
#[derive(Clone)]
pub struct DragPosition(pub Signal<(i32, i32)>);

/// Clicked position of the dragable element
#[derive(Clone)]
pub struct DragInitialOffset(pub Signal<(i32, i32)>);

/// Appends a layer onto an element to make it a drag focus.
#[component]
pub fn DraggableElement(children: Element) -> Element {
    let mut dragging = use_context::<Dragging>();

    let position_string = use_context::<PositionString>();
    let position = use_context::<DragPosition>();
    let mut initial_drag_offset = use_context::<DragInitialOffset>();

    let document = use_context::<AppDocument>();

    let onmousedown = move |evt: Event<MouseData>| {
        let keys = evt.held_buttons();
        if keys.contains(MouseButton::Primary) && keys.contains(MouseButton::Secondary) {
            // stop under elements from recieving clicks

            evt.stop_propagation();
            evt.prevent_default();

            // deselect all currently selected elements.
            document
                .0
                .read()
                .eval("window.getSelection().removeAllRanges();".to_string());

            // deselect text boxes
            document
                .0
                .read()
                .eval("document.activeElement?.blur()".to_string());

            // Get current element position
            let (left, top) = *position.0.read();
            // Compute offset from mouse to top-left corner
            let offset_x = evt.client_coordinates().x - left as f64;
            let offset_y = evt.client_coordinates().y - top as f64;

            initial_drag_offset
                .0
                .set((offset_x as i32, offset_y as i32));

            dragging.0.set(true);
        }
    };

    let result = rsx! {
        div {
            position: "relative",
            z_index: 998,
            onmousedown,
            oncontextmenu: |evt| {
                evt.prevent_default();
            },
            style: position_string.0,
            pointer_events: "auto",
            user_select: "none",
            {children}
        }
    };
    result
}

/// Component that renders a surface covering the given element that can be used as a drag surface for [`DraggableElement`] marked elements.
#[component]
pub fn DragSurfaceLayer(element: Result<VNode, RenderError>) -> Result<VNode, RenderError> {
    let mut dragging = use_context_provider(|| Dragging(Signal::new(false)));

    let initial_drag_offset = use_context_provider(|| DragInitialOffset(Signal::new((0, 0))));

    let mut position = use_context_provider(|| DragPosition(Signal::new((0, 0))));

    let mut mouse_on_screen = use_signal(|| true);

    let position_str = use_memo(move || {
        let position: (i32, i32) = *position.0.read();

        let new_pos: String = format!(
            "
            position: absolute;
            left: {}px; 
            top: {}px;
            ",
            position.0, position.1
        );
        return new_pos;
    });

    use_context_provider(|| PositionString(position_str));

    let onmousemove = move |evt: Event<MouseData>| {
        if *dragging.0.read() {
            let new_x = (evt.client_coordinates().x) - (initial_drag_offset.0.read().0) as f64;
            let new_y = (initial_drag_offset.0.read().1 as f64 + evt.client_coordinates().y)
                - (initial_drag_offset.0.read().1 * 2) as f64;
            position.0.set((new_x as i32, new_y as i32));
        }
    };

    let mut deselected_during_this_drag = use_signal(|| false);

    let onmouseup = move |evt: Event<MouseData>| {
        evt.prevent_default();

        dragging.0.set(false);
        mouse_on_screen.set(false);
        deselected_during_this_drag.set(false)
    };

    let onmouseleave = move |_evt: Event<MouseData>| {
        dragging.0.set(false);
        mouse_on_screen.set(false);
        deselected_during_this_drag.set(false)
    };

    let pointer_events = use_memo(move || {
        let status = match *dragging.0.read() {
            true => "auto",
            false => "none",
        };
        status
    });
    rsx!(
        div {

            {element}
            // child components are rendered ontop of parent components, so overlay is a child component so it renders ontop of the dragable element
            div {
                onmouseleave,
                onmousemove,
                onmouseup,

                pointer_events: pointer_events,
                z_index: 1000,
                width: "100%",
                height: "100%",
                position: "fixed",
                top: "0",
            }

        }
    )
}
