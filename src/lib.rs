//! # noon
//!
//! Strongly-typed, compile-time mediator.
//!
//! This crate allows you to use the [mediator pattern](https://en.wikipedia.org/wiki/Mediator_pattern) in Rust.
//! Unlike mediator implementations in most other languages, `noon`'s mediators are checked at compile time for correct registration.
//!
//! The design is partially inspired by the [MediatR](https://github.com/jbogard/MediatR) library for .NET.
//! ## Concepts
//! Components interact with this library through the [`mediator::Mediate`] trait.
//!
//! Mediators can be used to implement different behaviours by exchanging typed messages between components through different kinds of receivers.
//! There are two kinds of receiver:
//!
//! **Handlers** are used to perform some behaviour when presented a message of type `TMsg` and produce a response of type `TResp`.
//! A mediator typically has a single handler for a given type of message.
//!
//! **Notification receivers** are used to perform some behaviour when presented a message of type `TMsg` without returning anything in response.
//! A mediator might have multiple notification receivers for a given type of message, all of which are called in sequence whenever a message of that type is presented to the mediator.
//!
//! In noon, both handlers are notification receivers may be either synchronous or asynchronous.
//!
//! The types of messages that can be presented to the mediator are part of the mediator's type, including whether a type of message is able to be used to invoke a handler, send notifications, or both. Likewise, whether the receivers for a type of message are synchronous, asynchronous, or both is also tracked as part of the mediator's type.
//!
//! ## Conventions
//! In order to validate noon's registration constraints, this crate's public API has many generic parameters and constraint clauses on its signatures.
//!
//! To help maintain legibility, a few conventions are used with the names of generic parameters to suggest how they should be interpreted and used.
//!
//! `TMsg` is the type of a message the mediator should accept.
//!
//! `TResp` is the type of a response produced by a handler for a message.
//!
//! `F` and `Fut` are opaque, unique types for functions and futures respectively. These should generally be inferred by the compiler, not supplied by the user.
//!
//! `I` is a type-level index into the type-level list of registered message handlers or notification receivers. If your mediator only has at most a single receiver (or set of receivers in the case of notifications) for each registered message type and each sync/async kind, the compiler should automatically infer the correct type-level index for this parameter. If you find yourself in a position where you've only registered at most one kind of receiver for a message type and you have to manually provide the index, something is probably wrong.
//!
//! ## Usage with generic constraints
//! If you'd like to use a generic mediator and place constraints on the kinds of receivers it must contain, you're likely looking to require the [`hlist::ContainsAt<T, I>`] trait on the [`mediator::Mediate`]'s associated `Handlers` or `NotificationReceivers` associated types.
//!
//! These type-level lists implement different traits with generics populated from the [`entry`] module depending on the receivers that are registered with the mediator.
//! For example, a mediator with a synchronous handler accepting a `NewUserRequest` and producing a `NewUserResponse` would have an [`entry::RequestResponse<NewUserRequest,NewUserResponse>`] in its associated `Handlers` type-level list. Concretely, this means the associated `Handlers` type implements [`hlist::ContainsAt<entry:RequestResponse<NewUserRequest,NewUserResponse>, I>`] for some `I`.
//! ## Example
//! You can create a mediator using a builder interface. The following creates a mediator without any receivers.
//! ```rust
//! use noon::mediator::{Mediate, MediatorBuilder};
//!
//! let mediator = MediatorBuilder::new()
//!     .build();
//! ```
//! You can register synchronous or asynchronous handlers
//! ```rust
//! use noon::mediator::{Mediate, MediatorBuilder};
//!
//! struct NewUserMessage { id: i32 }
//! struct SendUserEmail { id: i32, msg: String };
//!
//! async fn foo() {
//!     let mediator = MediatorBuilder::new()
//!         .add_handler(|req: NewUserMessage| {
//!             println!("Hi, user {}!", req.id);
//!             req.id
//!         })
//!         .add_async_handler(|req: SendUserEmail| async move {
//!             // call email service
//!             true // return some response
//!         })
//!         .build();
//!     // prints "Hi, user 5!"
//!     let result = mediator.handle(NewUserMessage { id: 5 });
//!     assert_eq!(result, 5);
//!     let result2 = mediator.handle_async(SendUserEmail {
//!         id: 5, msg: "Hi!".to_string()
//!     }).await;
//! }
//! ```
//! You can't ask a mediator to handle a message it doesn't have a receiver for.
//! ```rust,compile_fail
//! use noon::mediator::{Mediate, MediatorBuilder}
//!
//! struct NewUserMessage { id: i32 }
//!
//! let mediator = MediatorBuilder::new()
//!     .build();
//! // Compile-time error, this mediator has no handler for that message.
//! mediator.handle(NewUserMessage { id: 10 });
//! ```
//! You can register multiple notification receivers for a single message type
//! ```rust
//! use noon::mediator::{Mediate, MediatorBuilder};
//!
//! #[derive(Debug)]
//! struct NewUserMessage { id: i32 }
//!
//! let mediator = MediatorBuilder::new()
//!     .listen_for::<NewUserMessage>()
//!     .add_notification_receiver(|msg: &NewUserMessage| {
//!         println!("New user {:?} created", msg);
//!     })
//!     .add_notification_receiver(|msg: &NewUserMessage| {
//!         // get total users in the system
//!         let total = 42;
//!         println!("The system now contains {} users", total);
//!     })
//!     .build();
//! // prints both messages in sequence
//! mediator.notify(&NewUserMessage { id: 5 });
//! ```
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

#[cfg(doctest)]
mod external_doctests {
    #[doc = include_str!("../README.md")]
    struct Null;
}
