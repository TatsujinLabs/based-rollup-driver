use std::fmt::Debug;

use async_trait::async_trait;
use tokio_util::sync::WaitForCancellationFuture;

/// The communication context used by the actor.
pub trait CancellableContext: Send {
    /// Returns a future that resolves when the actor is cancelled.
    fn cancelled(&self) -> WaitForCancellationFuture<'_>;
}

/// The [NodeActor] is an actor-like service for the node.
///
/// Actors may:
/// - Handle incoming messages.
///     - Perform background tasks.
/// - Emit new events for other actors to process.
#[async_trait]
pub trait DriverActor {
    type Error: Debug;
    type Inbond: Sized;
    type Outbond: CancellableContext;
    type Config;

    /// Builds the actor.
    fn build(config: Self::Config) -> (Self::Inbond, Self);

    /// Starts the actor.
    async fn start(self, outbond: Self::Outbond) -> Result<(), Self::Error>;
}
