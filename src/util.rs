use syn::{
    punctuated::Punctuated, token::Comma, DeriveInput, Field, GenericArgument, Ident,
    PathArguments, Type,
};

// Return whether a type matches a given &str
fn is_type(type_as_str: &str, ty: &syn::Type) -> bool {
    if let syn::Type::Path(ref p) = ty {
        return p.path.segments.len() == 1 && p.path.segments[0].ident == type_as_str;
    } else {
        false
    }
}

// Return whether a field has a given type
fn field_has_type(type_as_str: &str, field: &syn::Field) -> bool {
    is_type(type_as_str, &field.ty)
}

// Return whether a field is an optionized type
fn field_is_optionized(type_as_str: &str, field: &syn::Field) -> bool {
    if field_is_option(field) {
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

        is_type(type_as_str, &ty)
    } else {
        false
    }
}

pub fn field_is_string(field: &syn::Field) -> bool {
    field_has_type("String", field)
}

pub fn field_is_bool(field: &syn::Field) -> bool {
    field_has_type("bool", field)
}

pub fn field_is_option(field: &syn::Field) -> bool {
    field_has_type("Option", field)
}

pub fn field_is_option_string(field: &syn::Field) -> bool {
    field_is_optionized("String", field)
}

pub fn field_is_option_bool(field: &syn::Field) -> bool {
    field_is_optionized("bool", field)
}

/// Produce a new Ident by appending to the string verison, i.e.
/// Name -> NameBuilder etc.
pub fn append_to_ident(ident: &Ident, to_append: &str) -> Ident {
    let new_name = format!("{}{}", ident, to_append);
    syn::Ident::new(&new_name, ident.span())
}

/// Get the fields of a struct represented as a derive input
pub fn get_struct_fields(ast: &DeriveInput) -> Punctuated<Field, Comma> {
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named.clone()
    } else {
        panic!("YForm can only be derived for structs with named fields");
    };
    fields
}
