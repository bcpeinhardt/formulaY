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
//! - [ ] Auto applied classes for required fields after submit attempt
//! - [ ] Clean up how user imports requirements
//! 
//! # Example
//! ```
//! use formula_y::YForm;
//! use gloo::console::log;
//! use wasm_bindgen::JsCast;
//! use web_sys::HtmlInputElement;
//! 
//! use yew::prelude::*;
//! 
//! #[derive(Debug, Clone, YForm)]
//! pub struct Data {
//!     pub email: String,
//!     pub agree_to_terms: bool,
//! }
//! 
//! #[function_component(Index)]
//! pub fn index() -> Html {
//! 
//!    let onsubmit = Callback::from(|data: Data| {
//!         let msg = format!("Data succesfully passed! {:?}", data);
//!         log!(msg);
//!    });
//! 
//!    html! { <DataForm {onsubmit} /> }
//! }
//! 
//! fn main() {
//!    yew::start_app::<Index>();
//! }
//! ```
//! 
//! This produces the following html
//! ```html
//! <form class="data-form formula-y-form">
//!     <label class="email-label formula-y-txt-label">Email</label>
//!     <input type="text" class="email-input formula-y-txt-input">
//!     <label class="agree-to-terms-label formula-y-checkbox-label">Agree To Terms</label>
//!     <input type="checkbox" class="agree-to-terms-input formula-y-checkbox">
//!     <button>Submit</button>
//! </form>
//! ```
//! 
//! # How
//! Basically, the form will maintain an instance of the struct where each value is equal to the current input 
//! value of the form. Then the user can provide an onsubmit function as a `Callback<T>` where `T` 
//! is the type the form is derived from for the onsubmit. For instance,
//! said function might make a POST request with the struct as the request body.
//! 
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


use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Type, PathArguments, GenericArgument};
use convert_case::{Case, Casing};

#[proc_macro_derive(YForm)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // Get the Ident of the struct being derived
    let name = &ast.ident;

    // Generate the Component Ident
    let component_name = format!("{}Form", name);
    let component_ident = syn::Ident::new(&component_name, name.span());

    // Generate the Message Ident
    let component_msg_name = format!("{}Msg", component_ident);
    let component_msg_ident = syn::Ident::new(&component_msg_name, name.span());

    // Generate the Prop Ident
    let component_prop_name = format!("{}Props", component_ident);
    let component_prop_ident = syn::Ident::new(&component_prop_name, name.span());

    // Get the fields of the struct (Not implemented for Enums or TupleStructs)
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        panic!("YForm can only be derived for structs with named fields");
    };

    // Convenience functions for checking the type of a field
    let ty_is = |type_as_str: &str, field: &syn::Field| {
        if let syn::Type::Path(ref p) = field.ty {
            return p.path.segments.len() == 1 && p.path.segments[0].ident == type_as_str;
        }
        false
    };

    let ty_is_string = |field: &syn::Field| ty_is("String", field); 
    let ty_is_bool = |field: &syn::Field| ty_is("bool", field);
    let ty_is_option = |field: &syn::Field| ty_is("Option", field); 

    let ty_is_optionized = |type_as_str: &str, field: &syn::Field | {
        if ty_is_option(field) {
            let ty = match field.ty.clone() {
                Type::Path(typepath) if typepath.qself.is_none() => {
                    // Get the first segment of the path (there is only one, in fact: "Option"):
                    let type_params = typepath.path.segments[0].arguments.clone();
                    // It should have only on angle-bracketed param ("<String>"):
                    let generic_arg = match type_params {
                        PathArguments::AngleBracketed(params) => params.args[0].clone(),
                        _ => panic!("TODO: error handling"),
                    };
                    // This argument must be a type:
                    match generic_arg {
                        GenericArgument::Type(ty) => ty,
                        _ => panic!("TODO: error handling"),
                    }
                }
                _ => panic!("TODO: error handling"),
            };

            if let syn::Type::Path(ref p) = ty {
                return p.path.segments.len() == 1 && p.path.segments[0].ident == type_as_str;
            }
            false
        } else {
            false
        }
    }; 

    let ty_is_optionized_string = |field: &syn::Field| ty_is_optionized("String", field);
    let ty_is_optionized_bool = |field: &syn::Field| ty_is_optionized("bool", field);

    // Create the fields for initializing the struct
    let component_field_inits = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        if ty_is_string(field) {
            quote! { #field_ident: String::new() }
        } else if ty_is_bool(field) {
            quote! { #field_ident: false }
        } else if ty_is_option(field) {
            quote! { #field_ident: None }
        } else {
            panic!("Field type not supported");
        }
        
        
    });

    // Create the msg variants for updating each field
    let msg_variants = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        let field_type = field.ty.clone();

        let msg_variant = format!("update_{}", field_ident).to_case(Case::UpperCamel);
        let msg_variant_ident = syn::Ident::new(&msg_variant, name.span());

        quote! { #msg_variant_ident(#field_type) }
    });

    // Create the match arms for the update fn
    let match_arms_update = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();

        let msg_variant = format!("update_{}", field_ident).to_case(Case::UpperCamel);
        let msg_variant_ident = syn::Ident::new(&msg_variant, name.span());

        quote! { #component_msg_ident::#msg_variant_ident(item) => {
            self.inner.#field_ident = item;
            false
        } }
    });

    // Get the required fields 
    let required_string_fields: Vec<&syn::Field> = fields.iter().filter(|field| ty_is_string(field)).collect();
    let checks = required_string_fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        quote! {
            if self.inner.#field_ident == "" {
                return false;
            }
        }
    });

    let required_bool_fields: Vec<&syn::Field> = fields.iter().filter(|field| ty_is_bool(field)).collect();
    let bool_checks = required_bool_fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        quote! {
            if !self.inner.#field_ident {
                return false;
            }
        }
    });

    // Create the actual html elements for the inside of the form for the view fn
    let form_fields = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();

        let msg_variant = format!("update_{}", field_ident).to_case(Case::UpperCamel);
        let msg_variant_ident = syn::Ident::new(&msg_variant, name.span());

        let label = format!("{}", field_ident).to_case(Case::Title);

        if ty_is_string(field) {

            let label_class = format!("{} formula-y-txt-label required", format!("{}-label", field_ident).to_case(Case::Kebab));
            let input_class = format!("{} formula-y-txt-input required", format!("{}-input", field_ident).to_case(Case::Kebab));

            quote! {
                if self.inner.#field_ident == "" && self.display_required_warnings {
                    <span style="color: red;" class="formul-y-required-asterisk">{"*"}</span>
                }
                <label class={#label_class}>{#label}</label>
                <input class={#input_class} type="text" onchange={ctx.link().callback(move |event: Event| {
                    let new_value = event
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>()
                        .value();
    
                    #component_msg_ident::#msg_variant_ident(new_value)
                })} />
            }
        } else if ty_is_bool(field) {

            let label_class = format!("{} formula-y-checkbox-label required", format!("{}-label", field_ident).to_case(Case::Kebab));
            let input_class = format!("{} formula-y-checkbox required", format!("{}-input", field_ident).to_case(Case::Kebab));

            quote! {
                if !self.inner.#field_ident && self.display_required_warnings {
                    <span style="color: red;">{"*"}</span>
                }
                <label class={#label_class}>{#label}</label>
                <input class={#input_class} type="checkbox" checked={self.inner.#field_ident} onchange={ctx.link().callback(move |event: Event| {
                    let new_value = event
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>()
                        .checked();
    
                    #component_msg_ident::#msg_variant_ident(new_value)
                })} />
            }
        } else if ty_is_optionized_string(field) {
            let label_class = format!("{} formula-y-txt-label", format!("{}-label", field_ident).to_case(Case::Kebab));
            let input_class = format!("{} formula-y-txt-input", format!("{}-input", field_ident).to_case(Case::Kebab));

            quote! {
                <label class={#label_class}>{#label}</label>
                <input class={#input_class} type="text" onchange={ctx.link().callback(move |event: Event| {
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
            }
        } else if ty_is_optionized_bool(field) {

            let label_class = format!("{} formula-y-checkbox-label", format!("{}-label", field_ident).to_case(Case::Kebab));
            let input_class = format!("{} formula-y-checkbox", format!("{}-input", field_ident).to_case(Case::Kebab));

            quote! {
                <label class={#label_class}>{#label}</label>
                <input class={#input_class} type="checkbox" checked={self.inner.#field_ident.unwrap_or_default()} onchange={ctx.link().callback(move |event: Event| {
                    let new_value = event
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>()
                        .checked();
    
                    #component_msg_ident::#msg_variant_ident(Some(new_value))
                })} />
            }
        } else {
            quote! {
                <p>{"type not supported"}</p> 
            }
        }
    });

    let form_class = format!("{}-form formula-y-form", format!("{}", name).to_case(Case::Kebab));

    quote! {

        impl #name {
            pub fn new() -> Self {
                Self {
                    #(#component_field_inits,)*
                }
            }
        }

        pub struct #component_ident {
            inner: #name,
            display_required_warnings: bool,
            submitted: bool
        }

        impl #component_ident {
            pub fn required_components_provided(&self) -> bool {
                #(#checks)* 

                #(#bool_checks)* 

                true
            }
        }

        pub enum #component_msg_ident {
            #(#msg_variants,)*

            OnSubmit,
            DisplayRequiredWarnings
        }

        #[derive(PartialEq, Properties)]
        pub struct #component_prop_ident {
            pub onsubmit: Callback<#name>,
            pub init: Option<#name>
        }

        impl Component for #component_ident {
            type Message = #component_msg_ident;
            type Properties = #component_prop_ident;

            fn create(ctx: &Context<Self>) -> Self {

                let inner = if let Some(init) = &ctx.props().init {
                    init.clone()
                } else {
                    #name::new()
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

                        if self.required_components_provided() {
                            ctx.props().onsubmit.emit(self.inner.clone());
                            self.submitted = true;
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
