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

pub trait IntoOwnedChunks<T> {
    fn into_owned_chunks(self, chunk_size: usize) -> Vec<Vec<T>>;
}

impl<T: BaseModel> IntoOwnedChunks<T> for Vec<T> {
    fn into_owned_chunks(self, chunk_size: usize) -> Vec<Vec<T>> {
        let mut chunks = Vec::new();
        let mut iter = self.into_iter();

        while let Some(element) = iter.next() {
            let mut chunk = Vec::with_capacity(chunk_size);
            chunk.push(element);
            chunk.extend(iter.by_ref().take(chunk_size - 1));
            chunks.push(chunk);
        }

        chunks
    }
}
