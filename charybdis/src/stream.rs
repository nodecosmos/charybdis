use std::pin::Pin;
use std::task::{Context, Poll};

use crate::errors::CharybdisError;
use crate::model::BaseModel;
use futures::{Stream, StreamExt, TryStreamExt};
use scylla::client::pager::TypedRowStream;
use scylla::errors::NextRowError;

pub struct CharybdisModelStream<T: BaseModel + 'static> {
    inner: TypedRowStream<T>,
    query_string: &'static str,
}

impl<T: BaseModel> CharybdisModelStream<T> {
    pub(crate) fn query_string(&mut self, query_string: &'static str) {
        self.query_string = query_string;
    }
}

impl<T: BaseModel> From<TypedRowStream<T>> for CharybdisModelStream<T> {
    fn from(iter: TypedRowStream<T>) -> Self {
        CharybdisModelStream {
            inner: iter,
            query_string: "",
        }
    }
}

impl<T: BaseModel> Stream for CharybdisModelStream<T> {
    type Item = Result<T, CharybdisError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner
            .poll_next_unpin(cx)
            .map_err(|e| CharybdisError::NextRowError(self.query_string, e))
    }
}

impl<T: BaseModel> CharybdisModelStream<T> {
    pub async fn try_collect(self) -> Result<Vec<T>, CharybdisError> {
        let results: Result<Vec<T>, NextRowError> = self.inner.try_collect().await;

        results.map_err(|e| CharybdisError::NextRowError(self.query_string, e))
    }
}
