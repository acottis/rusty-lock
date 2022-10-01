//! This program creates a window in order to register for
//! WTS SESSION events such as lock screens. Once we see an event
//! We can run our arbitory code
mod wynapi;
use wynapi::*;

fn main() {
    // Enable logging
    tracing_subscriber::fmt().init();

    // Create a window for the events to be sent to
    let handle = create_window_ex_a().unwrap();

    // Register the window to recieve the events
    wts_register_session_notification(handle);

    // Handle session notifcation events
    while let Some(msg) = get_message_a(handle) {
        match msg {
            WtsState::Lock => {
                println!("User lock happened... execute your code here")
            }
            _ => {}
        }
    }

    // Cleanup when we are done
    wts_unregister_session_notification(handle);
}
