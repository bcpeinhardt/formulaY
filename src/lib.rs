//! This crate provides a macro for deriving a yew component from a custom struct which represents 
//! a set of form inputs. The desired mvp is to be able to
//! 
//! - [x] Support String fields as text input
//! - [x] Support bool fields as checkbox input
//! - [x] Support passing an onsubmit function as a prop
//! - [ ] Support for passing css styling as a prop
//! - [ ] Support for regex validation for String fields
//! 
//! Basically, the form will maintain an instance of the struct where each value is equal to the current input 
//! value of the form. Then the user can provide an onsubmit function with the signature
//! `fn(data: Data) -> ()` where `Data` is the type the form is derived from for the onsubmit. For instance,
//! said function might make a POST request with the struct as the request body.
//! 
//! One of the cool things about using Rust for web is that you can use the same language on the frontend and 
//! the backend, just like JavaScript. One of the driving use cases for this library is to define a struct one time in a 
//! common lib, and then use it both on the backend for setting up crud api endpoints and on the frontend for 
//! deriving forms from.
//! 
//! For an example of how the macro is intended to be used see examples/basic_form.rs. 
//! 
//! To see the produced 
//! html, run `trunk serve --open`. Try submitting the form and you should see a log message from the provided onsubmit
//! in the console.
//! 
//! To see the expanded yew code for the example, run `cargo expand --example basic_form`. 


use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
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

    // Create the fields for initializing the struct
    let component_field_inits = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        if ty_is_string(field) {
            quote! { #field_ident: String::new() }
        } else if ty_is_bool(field) {
            quote! { #field_ident: false }
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

    // Create the actual html elements for the inside of the form for the view fn
    let form_fields = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();

        let msg_variant = format!("update_{}", field_ident).to_case(Case::UpperCamel);
        let msg_variant_ident = syn::Ident::new(&msg_variant, name.span());

        let label = format!("{}", field_ident);

        if ty_is_string(field) {
            quote! {
                <label>{#label}</label>
                <input type="text" onchange={ctx.link().callback(move |event: Event| {
                    let new_value = event
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>()
                        .value();
    
                    #component_msg_ident::#msg_variant_ident(new_value)
                })} />
            }
        } else if ty_is_bool(field) {
            quote! {
                <label>{#label}</label>
                <input type="checkbox" checked={self.inner.#field_ident} onchange={ctx.link().callback(move |event: Event| {
                    let new_value = event
                        .target()
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>()
                        .checked();
    
                    #component_msg_ident::#msg_variant_ident(new_value)
                })} />
            }
        } else {
            quote! {
                <p>{"type not supported"}</p> 
            }
        }
    });

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
            submitted: bool
        }

        pub enum #component_msg_ident {
            #(#msg_variants,)*

            OnSubmit
        }

        #[derive(PartialEq, Properties)]
        pub struct #component_prop_ident {
            onsubmit: Callback<#name>
        }

        impl Component for #component_ident {
            type Message = #component_msg_ident;
            type Properties = #component_prop_ident;

            fn create(_ctx: &Context<Self>) -> Self {
                Self {
                    inner: #name::new(),
                    submitted: false
                }
            }

            fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {

                match msg {
                    #(#match_arms_update,)*

                    #component_msg_ident::OnSubmit => {
                        ctx.props().onsubmit.emit(self.inner.clone());
                        true
                    }
                }
            }

            fn view(&self, ctx: &Context<Self>) -> Html {

                let link = ctx.link();
                html! {
                    <form onsubmit={link.callback(|e: FocusEvent| {
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
