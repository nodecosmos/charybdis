use crate::model::Model;
use crate::query::{CharybdisQuery, QueryResultWrapper, QueryValue};

pub trait Insert<M: Model>: Model {
    fn insert(&self) -> CharybdisQuery<M, QueryResultWrapper>;
    fn insert_if_not_exists(&self) -> CharybdisQuery<M, QueryResultWrapper>;
}

impl<M: Model> Insert<M> for M {
    fn insert(&self) -> CharybdisQuery<M, QueryResultWrapper> {
        CharybdisQuery::new(Self::INSERT_QUERY, QueryValue::Ref(self))
    }

    fn insert_if_not_exists(&self) -> CharybdisQuery<M, QueryResultWrapper> {
        CharybdisQuery::new(Self::INSERT_IF_NOT_EXIST_QUERY, QueryValue::Ref(self))
    }
}

// pub trait InsertWithCallbacks<M: Model + Callbacks> {
//     fn insert_cb(&mut self) -> CharybdisQuery<M, InsertWithCb<M>>;
// }
//
// impl<M: Model + Callbacks> InsertWithCallbacks<M> for M {
//     fn insert_cb(&mut self) -> CharybdisQuery<M, InsertWithCb<M>> {
//         let cq = CharybdisQuery::new(Self::INSERT_QUERY, &self);
//         cq.model(self)
//     }
// }
//
// pub trait InsertWithExtCallbacks<M: Model + ExtCallbacks> {
//     fn insert_cb(&mut self) -> CharybdisQuery<M, InsertWithExtCb<M>>;
// }
//
// impl<M: Model + ExtCallbacks> InsertWithExtCallbacks<M> for M {
//     async fn insert_cb(&mut self) -> CharybdisQuery<M, InsertWithExtCb<M>> {
//         let cq = CharybdisQuery::new(Self::INSERT_QUERY, &self);
//         cq.model(self)
//     }
// }
