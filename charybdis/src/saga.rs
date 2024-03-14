use crate::callbacks::Callbacks;
use crate::errors::CharybdisError;
use crate::model::Model;
use futures::future::BoxFuture;
use futures::FutureExt;
use std::future::Future;

type Action<M> = BoxFuture<'static, Result<(), <M as Callbacks>::Error>>;

struct SagaStep<M: Model + Callbacks> {
    action: Action<M>,
    compensating_action: Option<Action<M>>,
}

impl<M: Model + Callbacks> SagaStep<M> {
    fn new<F>(action: F, compensating_action: Option<F>) -> Self
    where
        F: Future<Output = Result<(), M::Error>> + Send + 'static,
    {
        SagaStep {
            action: action.boxed(),
            compensating_action: compensating_action.map(|f| f.boxed()),
        }
    }
}

pub struct Saga<M: Model + Callbacks> {
    steps: Vec<SagaStep<M>>,
    compensating_actions: Vec<Option<Action<M>>>,
}

impl<M: Model + Callbacks> Saga<M> {
    pub fn new() -> Self {
        Saga {
            steps: vec![],
            compensating_actions: vec![],
        }
    }

    pub fn add_step<F>(&mut self, action: F, compensating_action: Option<F>)
    where
        F: Future<Output = Result<(), M::Error>> + 'static,
    {
        self.steps.push(SagaStep::new(action, compensating_action));
    }

    async fn execute_steps(&mut self) -> Result<(), M::Error> {
        let steps = self.steps.drain(..);

        for step in steps {
            self.compensating_actions.push(step.compensating_action);

            if let Err(e) = step.action.await {
                return Err(M::Error::from(CharybdisError::SagaError(format!(
                    "Failed saga: {:?}. Recovery Successful",
                    e
                ))));
            }
        }

        Ok(())
    }

    pub(crate) async fn execute_compensating_actions(&mut self) -> Result<(), M::Error> {
        let compensating_actions = self.compensating_actions.drain(..);

        for compensating_action in compensating_actions.rev() {
            if let Some(compensating_action) = compensating_action {
                if let Err(e) = compensating_action.await {
                    return Err(M::Error::from(CharybdisError::SagaRecoveryError(format!(
                        "Failed to recover from error: {:?}",
                        e
                    ))));
                }
            }
        }

        Ok(())
    }

    // Executes the saga, performing compensating actions for any step that fails.
    // Actions are drained, so saga can we populated again.
    // Compensating actions are preserved so they can be called if any action fails.
    pub async fn execute(&mut self) -> Result<(), M::Error> {
        let res = self.execute_steps().await;

        if let Err(e) = res {
            self.execute_compensating_actions().await?;

            return Err(e);
        }

        Ok(())
    }
}
