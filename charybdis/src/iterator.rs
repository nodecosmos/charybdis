use scylla::transport::session::TypedRowIter;

use crate::errors::CharybdisError;
use crate::model::BaseModel;

pub struct CharybdisModelIterator<T: BaseModel> {
    inner: TypedRowIter<T>,
    query_string: &'static str,
}

impl<T: BaseModel> CharybdisModelIterator<T> {
    pub(crate) fn query_string(&mut self, query_string: &'static str) {
        self.query_string = query_string;
    }
}

impl<T: BaseModel> From<TypedRowIter<T>> for CharybdisModelIterator<T> {
    fn from(iter: TypedRowIter<T>) -> Self {
        Self {
            inner: iter,
            query_string: "",
        }
    }
}

impl<T: BaseModel> Iterator for CharybdisModelIterator<T> {
    type Item = Result<T, CharybdisError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|row| row.map_err(|e| CharybdisError::FromRowError(self.query_string, e)))
    }
}
