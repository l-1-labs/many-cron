//! Define new task types and how to deserialize them

use many::ManyError;
use many::{message::ResponseMessage, server::module::ledger::SendArgs};
use many_client::ManyClient;
use serde::Deserialize;

use std::sync::Arc;

pub mod ledger;

/// Task parameters for different task types
#[derive(Clone, Deserialize)]
#[serde(tag = "endpoint", content = "params")]
pub enum Params {
    #[serde(alias = "ledger.send", deserialize_with = "ledger::from_cbor")]
    LedgerSend(SendArgs),
}

/// The base Task type
#[derive(Clone, Deserialize)]
pub struct Task {
    pub schedule: String,
    #[serde(flatten)]
    pub params: Params,
}

impl Task {
    /// Execute the task. The function executed will depends on the task parameter type.
    pub(crate) async fn execute(
        self,
        client: Arc<ManyClient>,
    ) -> Result<ResponseMessage, ManyError> {
        match self.params {
            Params::LedgerSend(p) => ledger::send(client, p).await,
        }
    }
}

/// A simple task collection
#[derive(Deserialize)]
pub struct Tasks {
    tasks: Vec<Task>,
}

/// Allow iterating on our task collection
impl IntoIterator for Tasks {
    type Item = Task;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.tasks.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{Params, Tasks};
    use many::{types::ledger::TokenAmount, Identity};
    use std::str::FromStr;

    #[test]
    fn deserialize_ledger_send() {
        let json = r#"
        {
            "tasks": [
                {
                    "schedule": "1/5 * * * * *",
                    "endpoint": "ledger.send",
                    "params": "{1: \"mag4ug633ael65rftbhxu3sy7flqqrpdws372cpbqqupmtmat4\", 2: 10, 3: \"mqbfbahksdwaqeenayy2gxke32hgb7aq4ao4wt745lsfs6wiaaaaqnz\"}"
                }
            ]
        }
        "#;

        let tasks: Tasks = serde_json::from_str(json).unwrap();
        let task = tasks.into_iter().next().unwrap();

        assert_eq!(task.schedule, "1/5 * * * * *");

        match task.params {
            Params::LedgerSend(p) => {
                assert_eq!(
                    p.to,
                    Identity::from_str("mag4ug633ael65rftbhxu3sy7flqqrpdws372cpbqqupmtmat4")
                        .unwrap()
                );
                assert_eq!(p.amount, TokenAmount::from(10u16));
                assert_eq!(
                    p.symbol,
                    Identity::from_str("mqbfbahksdwaqeenayy2gxke32hgb7aq4ao4wt745lsfs6wiaaaaqnz")
                        .unwrap()
                );
            }
        }
    }
}
