use std::future::Future;
use std::pin::Pin;

pub struct RequestResponse<TMsg, TResp> {
    cb: Box<dyn Fn(TMsg) -> TResp>,
}

impl<F, TMsg, TResp> From<F> for RequestResponse<TMsg, TResp>
where
    F: Fn(TMsg) -> TResp + 'static,
{
    fn from(f: F) -> Self {
        Self { cb: Box::new(f) }
    }
}

impl<TMsg, TResp> RequestResponse<TMsg, TResp> {
    pub fn call(&self, msg: TMsg) -> TResp {
        (self.cb)(msg)
    }
}

pub struct RequestResponseAsync<TMsg, TResp> {
    cb: Box<dyn Fn(TMsg) -> Pin<Box<dyn Future<Output = TResp>>>>,
}

impl<F, Fut, TMsg, TResp> From<F> for RequestResponseAsync<TMsg, TResp>
where
    Fut: Future<Output = TResp> + 'static,
    F: (Fn(TMsg) -> Fut) + 'static,
{
    fn from(f: F) -> Self {
        let f = move |msg| Box::pin(f(msg)) as _;
        Self { cb: Box::new(f) }
    }
}

impl<TMsg, TResp> RequestResponseAsync<TMsg, TResp> {
    pub fn call(&self, msg: TMsg) -> impl Future<Output = TResp> {
        (self.cb)(msg)
    }
}

pub struct ReceiveNotification<TMsg: ?Sized> {
    cbs: Vec<Box<dyn Fn(&TMsg)>>,
}

impl<TMsg: ?Sized> ReceiveNotification<TMsg> {
    pub fn new() -> Self {
        Self { cbs: vec![] }
    }

    pub fn add(&mut self, f: impl Fn(&TMsg) + 'static) {
        let cb = Box::new(f);
        self.cbs.push(cb)
    }

    pub fn call(&self, msg: &TMsg) {
        for cb in &self.cbs {
            cb(msg);
        }
    }
}

pub struct ReceiveNotificationAsync<TMsg: ?Sized> {
    cbs: Vec<Box<dyn Fn(TMsg) -> Pin<Box<dyn Future<Output = ()>>>>>,
}

impl<TMsg: Clone> ReceiveNotificationAsync<TMsg> {
    pub fn new() -> Self {
        Self { cbs: vec![] }
    }

    pub fn add<F, Fut>(&mut self, f: F)
    where
        Fut: Future<Output = ()> + 'static,
        F: Fn(TMsg) -> Fut + 'static,
    {
        let f = move |msg| Box::pin(f(msg)) as _;
        self.cbs.push(Box::new(f));
    }

    pub async fn call(&self, msg: TMsg) {
        for cb in &self.cbs {
            cb(msg.clone()).await;
        }
    }
}
