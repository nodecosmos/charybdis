use crate::model::BaseModel;

pub trait New: Default + BaseModel {
    fn new() -> Self {
        Self::default()
    }
}

impl<M: Default + BaseModel> New for M {}
