
use crate::symbol::Symbol;



#[derive(Debug, Clone)]
pub enum PropertyKey
{
    Str(&'static str),
    String(String),
    Symbol(Symbol)
}


impl PartialEq for PropertyKey
{
    fn eq(&self, other: &Self) -> bool
    {
        match (self, other)
        {
            (PropertyKey::Str(a), PropertyKey::Str(b))       => a == b,
            (PropertyKey::Str(a), PropertyKey::String(b))    => *a == b.as_str(),
            (PropertyKey::String(a), PropertyKey::Str(b))    => a.as_str() == *b,
            (PropertyKey::String(a), PropertyKey::String(b)) => a == b,
            (PropertyKey::Symbol(a), PropertyKey::Symbol(b)) => a == b,
            _                                                => false
        }
    }
}


impl Eq for PropertyKey
{
}


impl std::hash::Hash for PropertyKey
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H)
    {
        match self
        {
            PropertyKey::Str(s) =>
                {
                    0u8.hash(state);
                    s.hash(state);
                },

            PropertyKey::String(s) =>
                {
                    0u8.hash(state);
                    s.hash(state);
                },

            PropertyKey::Symbol(s) =>
                {
                    1u8.hash(state);
                    s.hash(state);
                }
        }
    }
}
