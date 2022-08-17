use formula_y::YForm;
use yew::prelude::*;

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use gloo::console::log;

#[derive(YForm, Debug, Clone)]
pub struct Data {
    pub email: String,
    pub agree_to_terms: bool
}

fn data_onsubmit(data: Data) {
    let msg = format!("Onsubmit succesfully passed! Can use data {:?}", data);
    log!(msg);
}

fn main() {
    let _form = html! {
        <DataForm />
    };
}

