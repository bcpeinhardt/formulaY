use formula_y::YForm;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use gloo::console::log;

use yew::prelude::*;

#[derive(YForm, Debug, Clone)]
pub struct Data {
    pub email: String,
    pub agree_to_terms: bool,
}

fn data_onsubmit(data: Data) {
    let msg = format!("Onsubmit succesfully passed! Can use data {:?}", data);
    log!(msg);
}

#[function_component(Index)]
pub fn index() -> Html {
    html! { <DataForm /> }
}

fn main() {
    yew::start_app::<Index>();
}