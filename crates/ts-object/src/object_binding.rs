
use std::sync::{ Arc, RwLock };

use async_trait::async_trait;

use crate::{ property_key::PropertyKey,
             object::{ ExecutableObject, ObjectRef, ObjectTrait, RuntimeResult },
             object_type::ObjectType,
             value::Value };



pub struct ObjectBinding<T>
{
    value: RwLock<T>
}


#[async_trait]
impl<T> ObjectTrait for ObjectBinding<T>
    where
        T: ObjectType + Send + Sync
{
    async fn get(&self, key: &PropertyKey, _receiver: Value) -> RuntimeResult<Value>
    {
        let value =
            {
                let object = self.value.read().unwrap();
                object.get_property(key)?
            };

        Ok(value.unwrap_or(Value::Undefined))
    }

    async fn set(&self, key: &PropertyKey, _receiver: Value, value: Value) -> RuntimeResult<()>
    {
        {
            let mut object = self.value.write().unwrap();
            object.set_property(key, value)?;
        }

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


//impl<T> ArrayTrait for ObjectBinding<T>
//    where
//        T: ObjectType + Send + Sync
//{
//    //
//}


//impl<T> HashTrait for ObjectBinding<T>
//    where
//        T: ObjectType + Send + Sync
//{
//    //
//}
