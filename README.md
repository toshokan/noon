# noon
[![Crates.io](https://img.shields.io/crates/v/noon)](https://crates.io/crates/noon)
[![Crates.io](https://img.shields.io/crates/l/noon)](https://crates.io/crates/noon)
[![docs.rs](https://img.shields.io/docsrs/noon)](https://docs.rs/noon)
[![GitHub Actions](https://github.com/toshokan/noon/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/toshokan/noon/actions)

Strongly-typed, compile-time mediator.

## Documentation

See the documentation [on docs.rs](https://docs.rs/noon)

## Example
```rust
use noon::mediator::{Mediate, MediatorBuilder};

// Create message types
struct NewUserRequest { id: i32 };
struct NewUserResponse { total_users: u32 };
struct SendUserEmail { id: i32, msg: String };
struct NewUserLogin { id: i32 };

async fn foo() {
    // Create a mediator
    let mediator = MediatorBuilder::new()
        .add_handler(|x: NewUserRequest| {
            // get total users
            NewUserResponse { total_users: 13 }
        })
        .add_async_handler(|x: SendUserEmail| async move {
            // send email
            true
        })
        .listen_for::<NewUserLogin>()
        .add_notification_receiver(|x: &NewUserLogin| {
            println!("User {} logged in!", x.id);
        })
        .build();
        
    mediator.notify(&NewUserLogin { id: 5 });
    let response = mediator.handle(NewUserRequest { id: 5 });
    println!("There are now {} users in the system", response.total_users);
}

let mediator = MediatorBuilder::new()
    .build();
```

## License

noon is dual licensed under the terms of the MIT or Apache-2.0 licenses.
You may choose whichever you prefer.
