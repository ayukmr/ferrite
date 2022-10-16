use std::time::{Instant, Duration};

pub struct Message {
    // message content
    message: Option<String>,

    // creation time
    timestamp: Option<Instant>,
}

impl Message {
    // create new message
    pub fn new(message: String) -> Self {
        Self {
            message:   Some(message),
            timestamp: None,
        }
    }

    // set message content
    pub fn set_message(&mut self, message: String) {
        self.message   = Some(message);
        self.timestamp = Some(Instant::now());
    }

    // get message content
    pub fn message(&mut self) -> Option<&String> {
        self.timestamp.and_then(|time| {
            // clear message after five seconds
            if time.elapsed() > Duration::from_secs(5) {
                self.message   = None;
                self.timestamp = None;
                None
            } else {
                let msg = self.message.as_ref().unwrap();
                Some(msg)
            }
        })
    }
}
