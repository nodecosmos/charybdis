use crate::errors::CharybdisError;
use crate::model::BaseModel;

pub struct CharybdisModelIterator<T: BaseModel> {
    inner: std::vec::IntoIter<T>,
    query_string: &'static str,
}

impl<T: BaseModel> CharybdisModelIterator<T> {
    pub(crate) fn query_string(&mut self, query_string: &'static str) {
        self.query_string = query_string;
    }
}

impl<T: BaseModel> From<Vec<T>> for CharybdisModelIterator<T> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            inner: vec.into_iter(),
            query_string: "",
        }
    }
}

impl<T: BaseModel> Iterator for CharybdisModelIterator<T> {
    type Item = Result<T, CharybdisError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Ok)
    }
}
