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

    // The onsubmit is the only required prop. It tells the form 
    // what to do when your user clicks the submit btn.
    let onsubmit = Callback::from(|data: Data| {
        let msg = format!("Data succesfully passed! {:?}", data);
        log!(msg);
    });

    // Optional props

    // You can set an initial value for the inner struct.
    // Useful for setting up forms with defaults already in place
    let init = Data {
        name: Some("Ben".to_string()),
        email: "test@gmail.com".to_string(),
        agree_to_terms: false,

        // Have this checked by default
        subscribe_to_updates: Some(true),
    };

    // I dont really have a good reason not to provide this option,
    // and I can see use cases where it increases the reusability
    // of forms.
    let enforce_required_fields = true;

    html! { <DataForm {onsubmit} {init} {enforce_required_fields} /> }
}

fn main() {
    yew::start_app::<Index>();
}
