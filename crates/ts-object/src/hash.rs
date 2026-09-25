
use async_trait::async_trait;

use ts_object_macros::Object;

use crate::object::ObjectTrait;



#[async_trait]
pub trait HashTableTrait: ObjectTrait
{
    //
}

#[derive(Object)]
pub struct HashTableObject
{
    //
}


#[async_trait]
impl HashTableTrait for HashTableObject
{
    //
}
