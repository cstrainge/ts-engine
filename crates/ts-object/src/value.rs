
use crate::{ big_int::BigInt, object::ObjectRef };



#[derive(Clone)]
pub enum Value
{
    Null,
    Undefined,
    Bool(bool),
    Number(f64),
    BigInt(BigInt),
    String(String),
    Object(ObjectRef)
}
