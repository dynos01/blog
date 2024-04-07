#![feature(try_blocks)]

mod editor;
mod scroll_to_top;
mod util;

use anyhow::{anyhow, Result};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console;
use yew::{html, Component, Context, Html, Renderer};

use crate::{editor::MarkdownRenderer, scroll_to_top::ScrollToTop, util::*};

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
    let app_element = get_element_by_id("app")?;

    Renderer::<App>::with_root(app_element).render();

    let url = get_window()?
        .location()
        .pathname()
        .map_err(|_| anyhow!("failed to get pathname"))?;

    if url == "/editor" {
        MarkdownRenderer::start()?;
    }

    Ok(())
}
