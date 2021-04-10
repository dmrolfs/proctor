use crate::Ack;
use std::path::PathBuf;
use tokio::sync::{broadcast, mpsc, oneshot};

pub type PolicyFilterApi<E> = mpsc::UnboundedSender<PolicyFilterCmd<E>>;
pub type PolicyFilterMonitor<T, E> = broadcast::Receiver<PolicyFilterEvent<T, E>>;

#[derive(Debug)]
pub enum PolicyFilterCmd<E> {
    ReplacePolicy {
        new_policy: PolicySource,
        tx: oneshot::Sender<Ack>,
    },
    AppendPolicy {
        policy: PolicySource,
        tx: oneshot::Sender<Ack>,
    },
    ResetPolicy(oneshot::Sender<Ack>),
    Inspect(oneshot::Sender<PolicyFilterDetail<E>>),
}

impl<E> PolicyFilterCmd<E> {
    pub fn replace_policy(new_policy: PolicySource) -> (PolicyFilterCmd<E>, oneshot::Receiver<Ack>) {
        let (tx, rx) = oneshot::channel();
        (Self::ReplacePolicy { new_policy, tx }, rx)
    }

    pub fn append_policy(policy: PolicySource) -> (PolicyFilterCmd<E>, oneshot::Receiver<Ack>) {
        let (tx, rx) = oneshot::channel();
        (Self::AppendPolicy { policy, tx }, rx)
    }

    pub fn reset_policy() -> (PolicyFilterCmd<E>, oneshot::Receiver<Ack>) {
        let (tx, rx) = oneshot::channel();
        (Self::ResetPolicy(tx), rx)
    }

    pub fn inspect() -> (PolicyFilterCmd<E>, oneshot::Receiver<PolicyFilterDetail<E>>) {
        let (tx, rx) = oneshot::channel();
        (Self::Inspect(tx), rx)
    }
}

#[derive(Debug)]
pub struct PolicyFilterDetail<E> {
    pub name: String,
    pub environment: Option<E>,
}

#[derive(Debug, Clone)]
pub enum PolicyFilterEvent<T, E> {
    EnvironmentChanged(Option<E>),
    ItemBlocked(T),
}

#[derive(Debug)]
pub enum PolicySource {
    String(String),
    File(PathBuf),
}

impl PolicySource {
    pub fn from_string<S: Into<String>>(policy: S) -> Self {
        Self::String(policy.into())
    }

    pub fn from_path(policy_path: PathBuf) -> Self {
        Self::File(policy_path)
    }

    pub fn load_into(&self, oso: &oso::Oso) -> crate::graph::GraphResult<()> {
        let result = match self {
            PolicySource::String(policy) => oso.load_str(policy.as_str()),
            PolicySource::File(policy) => oso.load_file(policy),
        };
        result.map_err(|err| err.into())
    }
}
