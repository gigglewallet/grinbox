use crate::client::GrinboxSubscriptionHandler;
use crate::error::Result;

pub trait GrinboxSubscriber {
	fn subscribe(&mut self, handler: Box<GrinboxSubscriptionHandler + Send>) -> Result<()>;
	fn unsubscribe(&self);
	fn is_running(&self) -> bool;
}
