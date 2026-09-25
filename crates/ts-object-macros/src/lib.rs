
use proc_macro::TokenStream;
use quote::quote;
use syn::{ parse_macro_input, DeriveInput };



#[proc_macro_derive(Object, attributes(property, object))]
pub fn derive_object(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as DeriveInput);

    derive_object_impl(input)
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}


fn derive_object_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream>
{
    let name = &input.ident;

    Ok(quote!
    {
        impl ::ts_object::object_type::ObjectType for #name
        {
            fn get_property(&self, _key: &::ts_object::property_key::PropertyKey)
                -> ::ts_object::object::RuntimeResult<Option<::ts_object::value::Value>>
            {
                Ok(None)
            }

            fn set_property(&mut self,
                            _key: &::ts_object::property_key::PropertyKey,
                            _value: ::ts_object::value::Value)
                            -> ::ts_object::object::RuntimeResult<bool>
            {
                Ok(false)
            }
        }
    })
}
