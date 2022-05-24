pub(crate) mod concrete;
pub mod entry;
pub mod fragment;
pub mod hlist;
pub mod mediator;

#[cfg(test)]
mod test {

    use super::*;
    use entry::RequestResponse;
    use hlist::ContainsAt;
    use mediator::{Mediate, MediatorBuilder};

    #[test]
    fn should_typecheck() {
        fn _typecheck<M, IntIndex, BoolIndex>(mediator: M)
        where
            M: Mediate,
            <M as Mediate>::Handlers: ContainsAt<RequestResponse<i32, ()>, IntIndex>,
            <M as Mediate>::Handlers: ContainsAt<RequestResponse<bool, ()>, BoolIndex>,
        {
            mediator.handle(12i32);
            mediator.handle(false);
        }
    }

    #[test]
    fn should_create() {
        let mediator = MediatorBuilder::new()
            .add_handler(|x: i32| {
                println!("i32: {}", x);
                18
            })
            .add_handler(|_x: bool| true)
            .add_handler(|_x: bool| "yes")
            .build();
        let result = mediator.handle(5);
        assert_eq!(result, 18);
    }

    #[test]
    fn should_notify() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let val = Arc::new(AtomicUsize::new(0));

        let handler_val = Arc::clone(&val);
        let mediator = MediatorBuilder::new()
            .add_handler(|x: i32| x)
            .listen_for::<i32>()
            .listen_for::<bool>()
            .add_notification_receiver(move |_x: &bool| {
                handler_val.fetch_add(1, Ordering::SeqCst);
            })
            .build();
        mediator.notify(&true);

        assert_eq!(val.load(Ordering::SeqCst), 1)
    }

    #[test]
    fn should_register_async() {
        let _ = async {
            let mediator = MediatorBuilder::new()
                .add_async_handler(|x: i32| async move { x })
                .build();
            mediator.handle_async(5).await;
        };
    }

    #[test]
    fn should_notify_async() {
        let _ = async {
            let mediator = MediatorBuilder::new()
                .listen_for_async::<bool>()
                .add_async_notification_receiver(|_x: bool| async move {})
                .build();
            mediator.notify_async(true).await;
        };
    }
}
