use crate::entry::{
    ReceiveNotification, ReceiveNotificationAsync, RequestResponse, RequestResponseAsync,
};
use crate::hlist::{ContainsAt, HList, HListExt, Cons, Nil};
use crate::concrete::Mediator;

use std::future::Future;
use std::pin::Pin;

pub trait Mediate {
    type Handlers: HList;
    type NotifyReceivers: HList;

    fn handle<TMsg, TResp, I>(&self, msg: TMsg) -> TResp
    where
        Self::Handlers: ContainsAt<RequestResponse<TMsg, TResp>, I>;

    fn handle_async<TMsg: 'static, TResp: 'static, I>(
        &self,
        msg: TMsg,
    ) -> Pin<Box<dyn Future<Output = TResp>>>
    where
        Self::Handlers: ContainsAt<RequestResponseAsync<TMsg, TResp>, I>;

    fn notify<TMsg: ?Sized, I>(&self, msg: &TMsg)
    where
        Self::NotifyReceivers: ContainsAt<ReceiveNotification<TMsg>, I>;

    fn notify_async<TMsg: Clone + 'static, I>(
        &self,
        msg: TMsg,
    ) -> Pin<Box<dyn Future<Output = ()> + '_>>
    where
        Self::NotifyReceivers: ContainsAt<ReceiveNotificationAsync<TMsg>, I>;
}

pub struct MediatorBuilder<H, N> {
    contents: H,
    receivers: N
}

impl MediatorBuilder<Nil, Nil> {
    pub fn new() -> Self {
	Self {
	    contents: Nil,
	    receivers: Nil
	}
    }
}

impl<H: HList, N: HList> MediatorBuilder<H, N> {
    pub fn add_handler<TMsg, TResp>(
        self,
        handler: impl Fn(TMsg) -> TResp + 'static,
    ) -> MediatorBuilder<Cons<RequestResponse<TMsg, TResp>, H>, N> {
        let rr = RequestResponse::from(handler);
        MediatorBuilder {
            contents: self.contents.push(rr),
	    receivers: self.receivers
        }
    }

    pub fn add_async_handler<TMsg, TResp, F, Fut>(
	self,
	handler: F
    ) -> MediatorBuilder<Cons<RequestResponseAsync<TMsg, TResp>, H>, N>
    where
	Fut: Future<Output = TResp> + 'static,
	F: Fn(TMsg) -> Fut + 'static
    {
	let rr = RequestResponseAsync::from(handler);
	MediatorBuilder {
	    contents: self.contents.push(rr),
	    receivers: self.receivers
	}
    }

    pub fn listen_for<TMsg: ?Sized>(self) -> MediatorBuilder<H, Cons<ReceiveNotification<TMsg>, N>> {
        let rn = ReceiveNotification::new();
        MediatorBuilder {
	    contents: self.contents,
            receivers: self.receivers.push(rn),
        }
    }

    pub fn listen_for_async<TMsg: Clone>(self) -> MediatorBuilder<H, Cons<ReceiveNotificationAsync<TMsg>, N>> {
        let rn = ReceiveNotificationAsync::new();
        MediatorBuilder {
	    contents: self.contents,
            receivers: self.receivers.push(rn),
        }
    }

    pub fn add_notification_receiver<TMsg: ?Sized, I>(
        mut self,
        receiver: impl Fn(&TMsg) + 'static,
    ) -> Self
    where
        N: ContainsAt<ReceiveNotification<TMsg>, I>,
    {
        let receiver_set = self.receivers.take_mut();
        receiver_set.add(receiver);
        self
    }

    pub fn add_async_notification_receiver<TMsg: Clone, I, F, Fut>(
	mut self,
	receiver: F
    ) -> Self
    where
	N: ContainsAt<ReceiveNotificationAsync<TMsg>, I>,
	Fut: Future<Output = ()> + 'static,
	F: Fn(TMsg) -> Fut + 'static,
    {
	let receiver_set = self.receivers.take_mut();
        receiver_set.add(receiver);
        self
    }

    pub fn build(self) -> impl Mediate<Handlers = H, NotifyReceivers = N> {
        Mediator::new(self.contents, self.receivers)
    }
}
