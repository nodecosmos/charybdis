use crate::errors::CharybdisError;
use crate::model::BaseModel;
use futures::{Stream, StreamExt, TryStreamExt};
use scylla::transport::iterator::{NextRowError, TypedRowIterator};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct CharybdisModelStream<T: BaseModel> {
    inner: TypedRowIterator<T>,
}

impl<T: BaseModel> From<TypedRowIterator<T>> for CharybdisModelStream<T> {
    fn from(iter: TypedRowIterator<T>) -> Self {
        CharybdisModelStream { inner: iter }
    }
}

impl<T: BaseModel> Stream for CharybdisModelStream<T> {
    type Item = Result<T, CharybdisError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx).map_err(|e| CharybdisError::from(e))
    }
}

impl<T: BaseModel> CharybdisModelStream<T> {
    pub async fn try_collect(self) -> Result<Vec<T>, CharybdisError> {
        let results: Result<Vec<T>, NextRowError> = self.inner.try_collect().await;

        results.map_err(|e| CharybdisError::from(e))
    }
}
