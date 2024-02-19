use crate::errors::CharybdisError;
use crate::model::BaseModel;
use scylla::transport::session::TypedRowIter;

pub struct CharybdisModelIterator<T: BaseModel> {
    inner: TypedRowIter<T>,
}

impl<T: BaseModel> From<TypedRowIter<T>> for CharybdisModelIterator<T> {
    fn from(iter: TypedRowIter<T>) -> Self {
        Self { inner: iter }
    }
}

impl<T: BaseModel> Iterator for CharybdisModelIterator<T> {
    type Item = Result<T, CharybdisError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|row| row.map_err(|e| CharybdisError::from(e)))
    }
}
