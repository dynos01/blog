use anyhow::{anyhow, Result};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{console, window, Event, HtmlElement, ScrollBehavior, ScrollToOptions};
use yew::{html, Callback, Component, Context, Html};

pub struct ScrollToTop;

impl Component for ScrollToTop {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let onclick = Callback::from(|_| {
            if let Err(e) = on_click_impl() {
                console::error_1(&format!("Failed to scroll to top: {e}").into());
            }
        });

        // A button cannot be scrolled
        // so we listen to the scroll event on the window
        if let Err(e) = add_on_scroll() {
            console::error_1(&format!("Failed to add on scroll hook: {e}").into());
        }

        html! {
            <button {onclick} class="scroll-to-top"></button>
        }
    }
}

fn on_click_impl() -> Result<()> {
    let window = window().ok_or(anyhow!("failed to get window"))?;

    let mut options = ScrollToOptions::new();
    options.top(0.0);
    options.left(0.0);
    options.behavior(ScrollBehavior::Smooth);

    window.scroll_to_with_scroll_to_options(&options);

    Ok(())
}

fn add_on_scroll() -> Result<()> {
    let onscroll = Callback::from(|_| {
        if let Err(e) = on_scroll_impl() {
            console::error_1(&format!("Failed to adjust scroll to top button: {e}").into());
        }
    });

    let closure = Closure::wrap(Box::new(move |_: Event| {
        onscroll.emit(());
    }) as Box<dyn FnMut(_)>);

    window()
        .ok_or(anyhow!("failed to get window"))?
        .add_event_listener_with_callback("scroll", closure.as_ref().unchecked_ref())
        .map_err(|e| anyhow!("cannot add event listener: {e:?}"))?;

    closure.forget();

    Ok(())
}

fn on_scroll_impl() -> Result<()> {
    let window = window().ok_or(anyhow!("failed to get window"))?;

    let document = window.document().ok_or(anyhow!("failed to get document"))?;

    let document_element = document
        .document_element()
        .ok_or(anyhow!("failed to get document element"))?;

    let button = document
        .get_elements_by_class_name("scroll-to-top")
        .get_with_index(0)
        .ok_or(anyhow!("failed to find button"))?
        .dyn_into::<HtmlElement>()
        .map_err(|e| anyhow!("failed to process button: {e:?}"))?;

    let threshold = (document_element.scroll_height() / 2) as f64;

    let show = {
        let show: Result<bool> = try {
            let offset = window
                .scroll_y()
                .map_err(|e| anyhow!("cannot get scroll_y offset: {e:?}"))?;

            let viewport_height = window
                .inner_height()
                .map_err(|e| anyhow!("cannot get inner_height of window: {e:?}"))?
                .as_f64()
                .ok_or(anyhow!("cannot convert inner_height to f64"))?;

            if viewport_height > threshold {
                let threshold = viewport_height - threshold;
                offset > threshold
            } else {
                offset + viewport_height > threshold
            }
        };

        show.unwrap_or_else(|e| {
            let msg = format!("Failed to decide whether to show the scroll to top button: {e}");
            console::error_1(&msg.into());

            // In case of error, display anyway
            true
        })
    };

    let (opacity, pointer_events) = if show { ("1", "auto") } else { ("0", "none") };

    button
        .style()
        .set_property("opacity", opacity)
        .map_err(|e| anyhow!("cannot set style for button: {e:?}"))?;

    button
        .style()
        .set_property("pointer-events", pointer_events)
        .map_err(|e| anyhow!("cannot set style for button: {e:?}"))?;

    Ok(())
}
