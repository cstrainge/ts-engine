
use std::sync::{ Arc, RwLock };

use async_trait::async_trait;

use crate::{ property_key::PropertyKey,
             object::{ ExecutableObject, ObjectRef, ObjectTrait, RuntimeResult },
             object_type::ObjectType,
             value::Value };



pub struct ObjectBinding<T>
{
    _value: RwLock<T>
}


#[async_trait]
impl<T> ObjectTrait for ObjectBinding<T>
    where
        T: ObjectType + Send + Sync
{
    async fn get(&self, _key: &PropertyKey, _receiver: Value) -> RuntimeResult<Value>
    {
        Ok(Value::Undefined)
    }

    async fn set(&self, _key: &PropertyKey, _receiver: Value, _value: Value) -> RuntimeResult<()>
    {
        Ok(())
    }

    async fn get_prototype(&self) -> RuntimeResult<Option<ObjectRef>>
    {
        Ok(None)
    }

    async fn set_prototype(&self, _prototype: Option<ObjectRef>) -> RuntimeResult<()>
    {
        Ok(())
    }

    async fn as_executable(&self) -> RuntimeResult<Option<Arc<dyn ExecutableObject>>>
    {
        Ok(None)
    }
}
