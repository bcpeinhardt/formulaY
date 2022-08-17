use formula_y::YForm;
use gloo::console::log;
use yew::prelude::*;

#[derive(YForm, Debug)]
pub struct Data {
    pub email: String,
    pub agree_to_terms: bool,
}

#[function_component(Index)]
pub fn index() -> Html {
    html! { <DataForm /> }
}

fn main() {
    yew::start_app::<Index>();
}