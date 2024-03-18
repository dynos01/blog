#![feature(try_blocks)]

mod editor;
mod scroll_to_top;

use anyhow::{anyhow, Result};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{console, window};
use yew::{html, Component, Context, Html, Renderer};

use crate::{editor::MarkdownRenderer, scroll_to_top::ScrollToTop};

struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        true
    }

    fn changed(&mut self, _: &Context<Self>, _: &Self::Properties) -> bool {
        true
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <>
                <ScrollToTop />
            </>
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    if let Err(e) = start_impl() {
        console::error_1(&format!("Failed to initialize application: {e}").into());
    }
}

fn start_impl() -> Result<()> {
    let window = window().ok_or(anyhow!("failed to get window"))?;

    let document = window.document().ok_or(anyhow!("failed to get document"))?;

    let app_element = document
        .get_element_by_id("app")
        .ok_or(anyhow!("failed to get element with id \"app\""))?;

    Renderer::<App>::with_root(app_element).render();

    let url = window
        .location()
        .pathname()
        .map_err(|_| anyhow!("failed to get pathname"))?;

    if url == "/editor" {
        MarkdownRenderer::start()?;
    }

    Ok(())
}
