use crate::error::CoreError;

pub trait HasId {
    type Id;
    fn id(&self) -> &Self::Id;
}

pub trait Validatable {
    fn validate(&self) -> Result<(), CoreError>;
}
