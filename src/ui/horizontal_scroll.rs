use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_html::geometry::{WheelDelta, euclid::Vector2D};

/// Component for horizontal drag setup
#[component]
pub fn HorizontalScroll(children: Element) -> Element {
    let mut header_element = use_signal(|| None);
    let mut scroll_position = use_signal(|| 0);

    let scroll_step = use_signal(|| 500);
    let scroll_width_string = use_memo(move || scroll_step.to_string() + "px");

    let mut client_width = use_signal(|| 0);

    let onscroll = move |evt: Event<ScrollData>| {
        *scroll_position.write() = evt.scroll_left() as i32;
        *client_width.write() = evt.client_width();
    };

    // handle scrolling between entries
    let onwheel = move |evt: Event<WheelData>| async move {
        let header_element: Option<Rc<MountedData>> = header_element.read().cloned();
        let Some(header_element) = header_element else {
            return;
        };

        // don't proc a new event if animation isn't increment of the scroll step I.E: isn't finished scrolling.
        let remainder = *scroll_position.read() % *scroll_step.read();
        if remainder != 0 {
            return;
        }

        let delta_y = match evt.delta() {
            WheelDelta::Pixels(vector3_d) => vector3_d.y,
            WheelDelta::Lines(vector3_d) => vector3_d.y,
            WheelDelta::Pages(vector3_d) => vector3_d.y,
        };

        let new_pos = if delta_y.is_sign_positive() {
            Vector2D::new((*scroll_position.read() + *scroll_step.read()) as f64, 0.0)
        } else {
            Vector2D::new((*scroll_position.read() - *scroll_step.read()) as f64, 0.0)
        };
        let _ = header_element.scroll(new_pos, ScrollBehavior::Smooth).await;
    };

    let onmounted = move |evt: Event<MountedData>| header_element.set(Some(evt.data()));

    rsx! {
        div {
            overflow_x: "scroll",
            white_space: "nowrap",
            width: scroll_width_string,
            background: "white",
            onmounted,
            onwheel,
            onscroll,
            {children}
        }
    }
}
