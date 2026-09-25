
use crate::{ property_key::PropertyKey, object::RuntimeResult, value::Value };



pub trait ObjectType
{
    fn get_property(&self, key: &PropertyKey) -> RuntimeResult<Option<Value>>;
    fn set_property(&mut self, key: &PropertyKey, value: Value) -> RuntimeResult<bool>;
}
