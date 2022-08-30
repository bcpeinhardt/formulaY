use formula_y::YForm;
use gloo::console::log;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, YForm)]
pub struct Data {
    pub name: Option<String>,
    pub email: String,
    pub agree_to_terms: bool,
    pub subscribe_to_updates: Option<bool>,
}

#[function_component(Index)]
pub fn index() -> Html {

    let onsubmit = Callback::from(|data: Data| {
        let msg = format!("Data succesfully passed! {:?}", data);
        log!(msg);
    });

    // This part is optional, but sometimes you want to have 
    // some defaults already set. Chances are you'll pull this
    // from an api request as well.
    let init = Data {
        name: None,
        email: String::new(),
        agree_to_terms: false,

        // Have this checked by default
        subscribe_to_updates: Some(true),
    };

    html! { <DataForm {onsubmit} {init} /> }
}

fn main() {
    yew::start_app::<Index>();
}
