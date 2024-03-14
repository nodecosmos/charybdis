use crate::callbacks::Callbacks;
use crate::errors::CharybdisError;
use futures::future::LocalBoxFuture;
use futures::FutureExt;
use std::future::Future;

pub type SagaAction<Err> = LocalBoxFuture<'static, Result<(), Err>>;

pub struct SagaStep<M: Callbacks> {
    action: SagaAction<M::Error>,
    compensating_action: Option<SagaAction<M::Error>>,
}

pub struct Saga<M: Callbacks> {
    steps: Vec<SagaStep<M>>,
    compensating_actions: Vec<SagaAction<M::Error>>,
}

impl<M: Callbacks> Saga<M> {
    pub fn new() -> Self {
        Saga {
            steps: vec![],
            compensating_actions: vec![],
        }
    }

    pub fn add_step(
        &mut self,
        action: impl Future<Output = Result<(), M::Error>> + 'static,
        compensating_action: Option<impl Future<Output = Result<(), M::Error>> + 'static>,
    ) -> &mut Self {
        self.steps.push(SagaStep {
            action: action.boxed_local(),
            compensating_action: compensating_action.map(|f| f.boxed_local()),
        });

        self
    }

    async fn execute_steps(&mut self) -> Result<(), M::Error> {
        let steps = self.steps.drain(..);

        for step in steps {
            if let Some(compensating_action) = step.compensating_action {
                self.compensating_actions.push(compensating_action);
            }

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
            if let Err(e) = compensating_action.await {
                return Err(M::Error::from(CharybdisError::SagaRecoveryError(format!(
                    "Failed to recover from error: {:?}",
                    e
                ))));
            }
        }

        Ok(())
    }

    pub(crate) async fn execute(&mut self) -> Result<(), M::Error> {
        let res = self.execute_steps().await;

        if let Err(e) = res {
            self.execute_compensating_actions().await?;

            return Err(e);
        }

        Ok(())
    }
}
