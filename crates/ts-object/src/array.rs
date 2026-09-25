
use async_trait::async_trait;

use ts_object_macros::Object;

use crate::object::{ ObjectTrait };


#[async_trait]
pub trait ArrayTrait: ObjectTrait
{
    //
}


#[derive(Object)]
pub struct ArrayObject
{
    //
}


#[async_trait]
impl ArrayTrait for ArrayObject
{
    //
}
