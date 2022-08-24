use formula_y::YForm;
use gloo::console::log;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use yew::prelude::*;

#[derive(Debug, Clone, YForm)]
pub struct Data {
    pub email: String,
    pub agree_to_terms: bool,
}

fn data_onsubmit(data: Data) {
    let msg = format!("Data succesfully passed! {:?}", data);
    log!(msg);
}

#[function_component(Index)]
pub fn index() -> Html {
    html! { <DataForm /> }
}

fn main() {
    yew::start_app::<Index>();
}
