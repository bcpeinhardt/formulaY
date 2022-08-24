# formula_y

This crate provides a macro for deriving a yew component from a custom struct which represents
a set of form inputs. The desired mvp is to be able to

- [x] Support String fields as text input
- [x] Support bool fields as checkbox input
- [ ] Support passing an onsubmit function as a prop
- [ ] Support for passing css styling as a prop

For an example of how the macro is intended to be used see `examples/basic_form.rs`
