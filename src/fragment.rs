use crate::concrete::Mediator;
use crate::entry::{
    ReceiveNotification, ReceiveNotificationAsync, RequestResponse, RequestResponseAsync,
};
use crate::hlist::{Cons, ContainsAt, HList, HListExt, Nil};
use crate::mediator::Mediate;

use std::future::Future;

pub struct Fragment<H, N> {
    contents: H,
    receivers: N,
}

impl Fragment<Nil, Nil> {
    pub fn empty() -> Self {
        Self {
            contents: Nil,
            receivers: Nil,
        }
    }
}

impl<H: HList, N: HList> Fragment<H, N> {
    pub fn add_handler<TMsg, TResp>(
        self,
        handler: impl Fn(TMsg) -> TResp + 'static,
    ) -> Fragment<Cons<RequestResponse<TMsg, TResp>, H>, N> {
        let rr = RequestResponse::from(handler);
        Fragment {
            contents: self.contents.push(rr),
            receivers: self.receivers,
        }
    }

    pub fn add_async_handler<TMsg, TResp, F, Fut>(
        self,
        handler: F,
    ) -> Fragment<Cons<RequestResponseAsync<TMsg, TResp>, H>, N>
    where
        Fut: Future<Output = TResp> + 'static,
        F: Fn(TMsg) -> Fut + 'static,
    {
        let rr = RequestResponseAsync::from(handler);
        Fragment {
            contents: self.contents.push(rr),
            receivers: self.receivers,
        }
    }

    pub fn listen_for<TMsg: ?Sized>(self) -> Fragment<H, Cons<ReceiveNotification<TMsg>, N>> {
        let rn = ReceiveNotification::new();
        Fragment {
            contents: self.contents,
            receivers: self.receivers.push(rn),
        }
    }

    pub fn listen_for_async<TMsg: Clone>(
        self,
    ) -> Fragment<H, Cons<ReceiveNotificationAsync<TMsg>, N>> {
        let rn = ReceiveNotificationAsync::new();
        Fragment {
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

    pub fn add_async_notification_receiver<TMsg: Clone, I, F, Fut>(mut self, receiver: F) -> Self
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
