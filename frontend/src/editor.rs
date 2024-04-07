use anyhow::{anyhow, Result};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{console, Event, HtmlElement, HtmlTextAreaElement};
use yew::{html, Callback, Component, Context, Html, Renderer};

use crate::util::*;

pub struct MarkdownRenderer;

impl Component for MarkdownRenderer {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _: &Context<Self>) -> Html {
        if let Err(e) = add_on_input() {
            console::error_1(&format!("Failed to add on change hook: {e}").into());
        }

        html! {}
    }
}

impl MarkdownRenderer {
    pub fn start() -> Result<()> {
        let editor_element = get_element_by_id("editor-markdown")?;
        Renderer::<Self>::with_root(editor_element).render();
        Ok(())
    }
}

fn add_on_input() -> Result<()> {
    let on_input = Callback::from(|_| {
        if let Err(e) = on_input_impl() {
            console::error_1(&format!("Failed to process input data: {e}").into());
        }
    });

    let closure = Closure::wrap(Box::new(move |_: Event| {
        on_input.emit(());
    }) as Box<dyn FnMut(_)>);

    let input_box = get_element_by_id("editor-markdown")?
        .dyn_into::<HtmlElement>()
        .map_err(|e| anyhow!("failed to process input box: {e:?}"))?;

    input_box
        .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
        .map_err(|e| anyhow!("cannot add event listener: {e:?}"))?;

    closure.forget();

    Ok(())
}

fn on_input_impl() -> Result<()> {
    use pulldown_cmark::{html, Parser};

    let markdown_input = get_element_by_id("editor-markdown")?
        .dyn_into::<HtmlTextAreaElement>()
        .map_err(|e| anyhow!("failed to process input box: {e:?}"))?
        .value();

    let parser = Parser::new(&markdown_input);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    get_elements_by_class_name("markdown-body")?
        .get(0)
        .ok_or(anyhow!("failed to get markdown body"))?
        .set_inner_html(&html_output);

    Ok(())
}
