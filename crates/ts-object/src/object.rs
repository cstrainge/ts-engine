
use std::{ collections::HashMap, sync::{ Arc, RwLock } };

use async_trait::async_trait;

use crate::{ property_descriptor::PropertyDescriptor, property_key::PropertyKey, value::Value };



pub type RuntimeResult<T> = Result<T, RuntimeError>;


/**
 * Represents the kinds of errors that can occur during the execution of T/JS code.
 */
pub enum RuntimeError
{
}


/**
 * Represents the core contract for all T/JS objects in the system, or native structs exported to
 * the scripting system.
 */
#[async_trait]
pub trait ObjectTrait: Send + Sync
{
    async fn get(&self, key: &PropertyKey, receiver: Value) -> RuntimeResult<Value>;
    async fn set(&self, key: &PropertyKey, receiver: Value, value: Value) -> RuntimeResult<()>;
    async fn get_prototype(&self) -> RuntimeResult<Option<ObjectRef>>;
    async fn set_prototype(&self, prototype: Option<ObjectRef>) -> RuntimeResult<()>;
    async fn as_executable(&self) -> RuntimeResult<Option<ExecutableObjectRef>>;
}


#[async_trait]
pub trait ExecutableObject: ObjectTrait
{
    async fn execute(&self, this: Value, args: Vec<Value>) -> RuntimeResult<Value>;
}


pub type ObjectRef = Arc<dyn ObjectTrait>;


pub type ExecutableObjectRef = Arc<dyn ExecutableObject>;


pub struct Object
{
    _properties: RwLock<HashMap<PropertyKey, PropertyDescriptor>>,
    _prototype: RwLock<Option<ObjectRef>>
}


impl Object
{
    pub fn new() -> Self
    {
        Self
        {
            _properties: RwLock::new(HashMap::new()),
            _prototype: RwLock::new(None),
        }
    }

    pub fn from_prototype(prototype: ObjectRef) -> Self
    {
        Self
        {
            _properties: RwLock::new(HashMap::new()),
            _prototype: RwLock::new(Some(prototype)),
        }
    }
}


#[async_trait]
impl ObjectTrait for Object
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
