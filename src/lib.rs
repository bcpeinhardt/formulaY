//! This crate provides a macro for deriving a yew component from a custom struct which represents
//! a set of form inputs. The desired mvp is to be able to
//!
//! - [x] Support String fields as text input
//! - [x] Support bool fields as checkbox input
//! - [x] Support passing an onsubmit function as a prop
//! - [x] Support for initializing form with default values
//! - [x] Support for custom css styling
//! - [ ] Support for regex validation for String fields
//! - [ ] Support for number type fields with automatic parsing validation
//! - [x] Support for required and optional fields with Option type
//! - [x] Auto applied classes for required fields after submit attempt
//! - [ ] Clean up how user imports requirements
//!
//! # How
//! Basically, the form will maintain an instance of the struct where each value is equal to the current input
//! value of the form. Then the user can provide an onsubmit function as a `Callback<T>` where `T`
//! is the type the form is derived from for the onsubmit. For instance,
//! said function might make a POST request with the struct as the request body.
//!
//! # Why
//! One of the cool things about using Rust for web is that you can use the same language on the frontend and
//! the backend, just like JavaScript. One of the driving use cases for this library is to define a struct one time in a
//! common lib, and then use it both on the backend for setting up crud api endpoints and on the frontend for
//! deriving forms from.
//!
//! For an example of how the macro is intended to be used see usage/src/main.rs.
//!
//! To see the produced
//! html, run `trunk serve --open` from the usage directory. Try submitting the form and you should see a log message from the provided onsubmit
//! in the console.
//!
//! # Styling
//! For the moment, the easiest way to style the elements is to use the auto-generated classnames. Each field and label get specific class
//! names and general class names for hooking into.
//!
//! To see the expanded yew code for the example, run `cargo expand --bin usage`.

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use util::{
    append_to_ident, field_is_bool, field_is_option, field_is_option_bool, field_is_option_string,
    field_is_string, get_struct_fields,
};

// Utilities
mod util;

// Generate the MSG variants responsible for updating each field.
// first_name -> UpdateFirstName
fn get_update_field_msg_variant_ident(field: &syn::Field, span_ident: &syn::Ident) -> syn::Ident {
    let field_ident = field.ident.clone().unwrap();
    let msg_variant = format!("update_{}", field_ident).to_case(Case::UpperCamel);
    syn::Ident::new(&msg_variant, span_ident.span())
}

// 
fn get_label_and_input_classes(field_ident: &syn::Ident) -> (String, String, String, String) {
    let txt_label_class = format!(
        "{} formula-y-txt-label",
        format!("{}-label", field_ident).to_case(Case::Kebab)
    );
    let txt_input_class = format!(
        "{} formula-y-txt-input",
        format!("{}-input", field_ident).to_case(Case::Kebab)
    );
    let bool_label_class = format!(
        "{} formula-y-checkbox-label",
        format!("{}-label", field_ident).to_case(Case::Kebab)
    );
    let bool_input_class = format!(
        "{} formula-y-checkbox",
        format!("{}-input", field_ident).to_case(Case::Kebab)
    );
    (
        txt_label_class,
        txt_input_class,
        bool_label_class,
        bool_input_class,
    )
}

#[proc_macro_derive(YForm)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // We are producing a yew component based on the input struct, so we will need
    // idents for the component struct, its msg enum, and its prop struct.
    let input_struct_ident = &ast.ident;
    let component_ident = append_to_ident(input_struct_ident, "Form"); // Data -> DataForm
    let component_msg_ident = append_to_ident(&component_ident, "Msg"); // Data -> DataFormMsg
    let component_prop_ident = append_to_ident(&component_ident, "Props"); // Data -> DataFormProps

    // Get the fields of the struct (Not implemented for Enums or TupleStructs)
    let fields = get_struct_fields(&ast);

    // For convenience, we generate a standard new() method for the struct.
    // To do so, we iterate over the supported types and produce the appropriate line.
    let component_field_inits = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        if field_is_string(field) {
            quote! { #field_ident: String::new() }
        } else if field_is_bool(field) {
            quote! { #field_ident: false }
        } else if field_is_option(field) {
            quote! { #field_ident: None }
        } else {
            panic!("Field type not supported");
        }
    });

    // Create the msg variants for updating each field
    let msg_variants = fields.iter().map(|field| {
        let field_type = field.ty.clone();
        let msg_variant_ident = get_update_field_msg_variant_ident(field, input_struct_ident);
        quote! { #msg_variant_ident(#field_type) }
    });

    // Create the match arms for the update fn for updating each field
    let match_arms_update = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        let msg_variant_ident = get_update_field_msg_variant_ident(field, input_struct_ident);

        quote! { #component_msg_ident::#msg_variant_ident(item) => {
            self.inner.#field_ident = item;
            false
        } }
    });

    // We need to have a way to check if the required fields have all been provided, so we generate
    // a series of if checks to confirm string fields are not empty strings and checkboxes are
    // checked.
    let required_string_fields: Vec<&syn::Field> = fields
        .iter()
        .filter(|field| field_is_string(field))
        .collect();
    let checks = required_string_fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        quote! {
            if self.inner.#field_ident == "" {
                return false;
            }
        }
    });

    let required_bool_fields: Vec<&syn::Field> =
        fields.iter().filter(|field| field_is_bool(field)).collect();
    let bool_checks = required_bool_fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        quote! {
            if !self.inner.#field_ident {
                return false;
            }
        }
    });

    // Now we are generating methods thats give us the class attributes text for each field. If a form submit occurs
    // and a required field is empty/unchecked, it gets a class of required appended to it.
    let get_class_methods = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();

        let (txt_label_class, txt_input_class, bool_label_class, bool_input_class) =
            get_label_and_input_classes(&field_ident);

        let method_name_label = format!("get_class_for_{}_label", field_ident);
        let method_name_label_ident =
            syn::Ident::new(&method_name_label, input_struct_ident.span());
        let method_name_input = format!("get_class_for_{}", field_ident);
        let method_name_input_ident =
            syn::Ident::new(&method_name_input, input_struct_ident.span());

        if field_is_string(field) {
            quote! {
                pub fn #method_name_label_ident(&self) -> String {
                    match self.display_required_warnings && self.inner.#field_ident == "" {
                        true => {
                            let mut base_name = String::from(#txt_label_class);
                            base_name.push_str(" required");
                            base_name
                        },
                        false => #txt_label_class.to_string()
                    }
                }

                pub fn #method_name_input_ident(&self) -> String {
                    match self.display_required_warnings && self.inner.#field_ident == "" {
                        true => {
                            let mut base_name = String::from(#txt_input_class);
                            base_name.push_str(" required");
                            base_name
                        },
                        false => #txt_input_class.to_string()
                    }
                }
            }
        } else if field_is_bool(field) {
            quote! {
                pub fn #method_name_label_ident(&self) -> String {
                    match self.display_required_warnings && !self.inner.#field_ident {
                        true => {
                            let mut base_name = String::from(#bool_label_class);
                            base_name.push_str(" required");
                            base_name
                        },
                        false => #bool_label_class.to_string()
                    }
                }

                pub fn #method_name_input_ident(&self) -> String {
                    match self.display_required_warnings && !self.inner.#field_ident{
                        true => {
                            let mut base_name = String::from(#bool_input_class);
                            base_name.push_str(" required");
                            base_name
                        },
                        false => #bool_input_class.to_string()
                    }
                }
            }
        } else {
            quote! {}
        }
    });

    // Create the actual html elements for the inside of the form for the view fn
    let form_fields = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        let msg_variant_ident = get_update_field_msg_variant_ident(field, input_struct_ident);

        let label = format!("{}", field_ident).to_case(Case::Title);

        let (txt_label_class, txt_input_class, bool_label_class, bool_input_class) = get_label_and_input_classes(&field_ident);

        let method_name_label = format!("get_class_for_{}_label", field_ident);
        let method_name_label_ident = syn::Ident::new(&method_name_label, input_struct_ident.span());
        let method_name_input = format!("get_class_for_{}", field_ident);
        let method_name_input_ident = syn::Ident::new(&method_name_input, input_struct_ident.span());

        if field_is_string(field) {
            quote! {
                <div class="formula-y-form-item">
                <label class={self.#method_name_label_ident()}>{#label}</label>
                <input class={self.#method_name_input_ident()} type="text" value={self.inner.#field_ident.clone()} onchange={ctx.link().callback(move |event: Event| {
                    let new_value = event
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>()
                        .value();
    
                    #component_msg_ident::#msg_variant_ident(new_value)
                })} />
                </div>
            }
        } else if field_is_bool(field) {

            quote! {
                <div class="formula-y-form-item">
                <label class={self.#method_name_label_ident()}>{#label}</label>
                <input class={self.#method_name_input_ident()} type="checkbox" checked={self.inner.#field_ident} onchange={ctx.link().callback(move |event: Event| {
                    let new_value = event
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>()
                        .checked();
    
                    #component_msg_ident::#msg_variant_ident(new_value)
                })} />
                </div>
            }
        } else if field_is_option_string(field) {

            quote! {
                <div class="formula-y-form-item">
                <label class={#txt_label_class}>{#label}</label>
                <input class={#txt_input_class} type="text" value={self.inner.#field_ident.clone().unwrap_or_default()} onchange={ctx.link().callback(move |event: Event| {
                    let new_value = event
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>()
                        .value();
    
                    if new_value == "" {
                        #component_msg_ident::#msg_variant_ident(None)
                    } else {
                        #component_msg_ident::#msg_variant_ident(Some(new_value))
                    }
                })} />
                </div>
            }
        } else if field_is_option_bool(field) {

            quote! {
                <div class="formula-y-form-item">
                <label class={#bool_label_class}>{#label}</label>
                <input class={#bool_input_class} type="checkbox" checked={self.inner.#field_ident.unwrap_or_default()} onchange={ctx.link().callback(move |event: Event| {
                    let new_value = event
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>()
                        .checked();
    
                    #component_msg_ident::#msg_variant_ident(Some(new_value))
                })} />
                </div>
            }
        } else {
            quote! {
                <p>{"type not supported"}</p> 
            }
        }
    });

    let form_class = format!(
        "{}-form formula-y-form",
        format!("{}", input_struct_ident).to_case(Case::Kebab)
    );

    quote! {

        impl #input_struct_ident {
            pub fn new() -> Self {
                Self {
                    #(#component_field_inits,)*
                }
            }
        }

        pub struct #component_ident {
            inner: #input_struct_ident,
            display_required_warnings: bool,
            submitted: bool
        }

        impl #component_ident {
            pub fn required_components_provided(&self) -> bool {
                #(#checks)* 

                #(#bool_checks)* 

                true
            }

            #(#get_class_methods)*
        }

        pub enum #component_msg_ident {
            #(#msg_variants,)*

            OnSubmit,
            DisplayRequiredWarnings
        }

        #[derive(PartialEq, Properties)]
        pub struct #component_prop_ident {
            pub onsubmit: Callback<#input_struct_ident>,
            pub init: Option<#input_struct_ident>,
            pub enforce_required_fields: Option<bool>
        }

        impl Component for #component_ident {
            type Message = #component_msg_ident;
            type Properties = #component_prop_ident;

            fn create(ctx: &Context<Self>) -> Self {

                let inner = if let Some(init) = &ctx.props().init {
                    init.clone()
                } else {
                    #input_struct_ident::new()
                };

                Self {
                    inner,
                    submitted: false,
                    display_required_warnings: false
                }
            }

            fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {

                match msg {
                    #(#match_arms_update,)*

                    #component_msg_ident::OnSubmit => {

                        let enforce_required = ctx.props().enforce_required_fields.unwrap_or_else(|| true);

                        if self.required_components_provided() || !enforce_required {
                            ctx.props().onsubmit.emit(self.inner.clone());
                            self.submitted = true;
                            self.display_required_warnings = false;
                        } else {
                            ctx.link().send_message(#component_msg_ident::DisplayRequiredWarnings);
                        }
                        true
                    },
                    #component_msg_ident::DisplayRequiredWarnings => {
                        self.display_required_warnings = true;
                        true
                    }
                }
            }

            fn view(&self, ctx: &Context<Self>) -> Html {

                let link = ctx.link();
                html! {
                    <form class={#form_class} onsubmit={link.callback(|e: FocusEvent| {
                        e.prevent_default();

                        #component_msg_ident::OnSubmit
                    })}>

                        #(#form_fields)*

                        <button>{"Submit"}</button>
                    </form>
                }
            }
        }
    }
    .into()
}
