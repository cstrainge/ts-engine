
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
        #[::async_trait::async_trait]
        impl ::ts_object::object::ObjectTrait for #name
        {
            async fn get(&self,
                         _key: &::ts_object::property_key::PropertyKey,
                         _receiver: ::ts_object::value::Value)
                         -> ::ts_object::object::RuntimeResult<::ts_object::value::Value>
            {
                Ok(::ts_object::value::Value::Undefined)
            }

            async fn set(&self,
                         _key: &::ts_object::property_key::PropertyKey,
                         _receiver: ::ts_object::value::Value,
                         _value: ::ts_object::value::Value)
                         -> ::ts_object::object::RuntimeResult<()>
            {
                Ok(())
            }

            async fn get_prototype(&self) ->
                ::ts_object::object::RuntimeResult<Option<::ts_object::object::ObjectRef>>
            {
                Ok(None)
            }

            async fn set_prototype(&self, _prototype: Option<::ts_object::object::ObjectRef>)
                -> ::ts_object::object::RuntimeResult<()>
            {
                Ok(())
            }

            async fn as_executable(&self) ->
                ::ts_object::object::RuntimeResult<Option<::ts_object::object::ExecutableObjectRef>>
            {
                Ok(None)
            }
        }
    })
}
