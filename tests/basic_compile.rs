use formula_y::YForm;
use yew::prelude::*;

#[derive(YForm)]
pub struct Data {
    pub email: String,
    pub agree_to_terms: bool
}

fn main() {
    let _form = html! {
        <DataForm />
    };
}

