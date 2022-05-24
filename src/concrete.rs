use crate::entry::RequestResponse;
use crate::hlist::{ContainsAt, HList};
use crate::mediator::Mediate;

use std::future::Future;
use std::pin::Pin;

pub struct Mediator<H, N> {
    contents: H,
    receivers: N,
}

impl<H: HList, N: HList> Mediator<H, N> {
    pub(crate) fn new(contents: H, receivers: N) -> Self {
        Self {
            contents,
            receivers,
        }
    }
}

impl<H: HList, N: HList> Mediate for Mediator<H, N> {
    type Handlers = H;
    type NotifyReceivers = N;

    fn handle<TMsg, TResp, I>(&self, msg: TMsg) -> TResp
    where
        Self::Handlers: ContainsAt<RequestResponse<TMsg, TResp>, I>,
    {
        let handler = self.contents.take();
        handler.call(msg)
    }

    fn handle_async<TMsg: 'static, TResp: 'static, I>(
        &self,
        msg: TMsg,
    ) -> Pin<Box<dyn Future<Output = TResp>>>
    where
        Self::Handlers: ContainsAt<crate::entry::RequestResponseAsync<TMsg, TResp>, I>,
    {
        let handler = self.contents.take();
        Box::pin(handler.call(msg))
    }

    fn notify<TMsg: ?Sized, I>(&self, msg: &TMsg)
    where
        Self::NotifyReceivers: ContainsAt<crate::entry::ReceiveNotification<TMsg>, I>,
    {
        let receivers = self.receivers.take();
        receivers.call(msg)
    }

    fn notify_async<TMsg: Clone + 'static, I>(
        &self,
        msg: TMsg,
    ) -> Pin<Box<dyn Future<Output = ()> + '_>>
    where
        Self::NotifyReceivers: ContainsAt<crate::entry::ReceiveNotificationAsync<TMsg>, I>,
    {
        let receivers = self.receivers.take();
        Box::pin(receivers.call(msg))
    }
}
