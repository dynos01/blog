use anyhow::{anyhow, Result};
use web_sys::{window, Document, Element, Window};

pub fn get_window() -> Result<Window> {
    window().ok_or(anyhow!("failed to get window"))
}

pub fn get_document() -> Result<Document> {
    get_window()?
        .document()
        .ok_or(anyhow!("failed to get document"))
}

pub fn get_element_by_id(id: &str) -> Result<Element> {
    get_document()?
        .get_element_by_id(id)
        .ok_or(anyhow!("failed to get element with id \"{}\"", id))
}

pub fn get_elements_by_class_name(class: &str) -> Result<Vec<Element>> {
    let elements = get_document()?.get_elements_by_class_name(class);

    let elements: Vec<_> = (0..elements.length())
        .filter_map(|i| elements.get_with_index(i))
        .collect();

    Ok(elements)
}
