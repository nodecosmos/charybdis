use crate::model::BaseModel;

pub trait New<T: Default + BaseModel> {
    fn new() -> Self;
}

impl<T: Default + BaseModel> New<T> for T {
    fn new() -> Self {
        Self::default()
    }
}
