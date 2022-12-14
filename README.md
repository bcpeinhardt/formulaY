# formula_y

This crate provides a macro for deriving a yew component from a custom struct which represents
a set of form inputs. The desired mvp is to be able to

- [x] Support String fields as text input
- [x] Support bool fields as checkbox input
- [x] Support passing an onsubmit function as a prop
- [x] Support for initializing form with default values
- [x] Support for custom css styling
- [ ] Support for regex validation for String fields
- [ ] Support for number type fields with automatic parsing validation
- [x] Support for required and optional fields with Option type
- [ ] Auto applied classes for required fields after submit attempt
- [ ] Clean up how user imports requirements

## How
Basically, the form will maintain an instance of the struct where each value is equal to the current input
value of the form. Then the user can provide an onsubmit function as a `Callback<T>` where `T`
is the type the form is derived from for the onsubmit. For instance,
said function might make a POST request with the struct as the request body.

## Why
One of the cool things about using Rust for web is that you can use the same language on the frontend and
the backend, just like JavaScript. One of the driving use cases for this library is to define a struct one time in a
common lib, and then use it both on the backend for setting up crud api endpoints and on the frontend for
deriving forms from.

For an example of how the macro is intended to be used see usage/src/main.rs.

To see the produced
html, run `trunk serve --open` from the usage directory. Try submitting the form and you should see a log message from the provided onsubmit
in the console.

## Styling
For the moment, the easiest way to style the elements is to use the auto-generated classnames. Each field and label get specific class
names and general class names for hooking into.

To see the expanded yew code for the example, run `cargo expand --bin usage`.
