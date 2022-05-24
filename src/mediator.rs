use crate::entry::{
    ReceiveNotification, ReceiveNotificationAsync, RequestResponse, RequestResponseAsync,
};
use crate::fragment::Fragment;
use crate::hlist::{ContainsAt, HList, Nil};

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

pub struct MediatorBuilder {}

impl MediatorBuilder {
    pub fn new() -> Fragment<Nil, Nil> {
        Fragment::empty()
    }
}
