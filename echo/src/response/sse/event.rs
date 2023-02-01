use std::{borrow::Cow, fmt, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Event {
    comment: Option<Cow<'static, str>>,
    retry: Option<Duration>,
    id: Option<Cow<'static, str>>,
    event: Option<Cow<'static, str>>,
    data: Option<Cow<'static, str>>,
}

impl Event {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn comment(mut self, comment: impl Into<Cow<'static, str>>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn retry(mut self, retry: Duration) -> Self {
        self.retry = Some(retry);
        self
    }

    pub fn id(mut self, id: impl Into<Cow<'static, str>>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn event(mut self, event: impl Into<Cow<'static, str>>) -> Self {
        self.event = Some(event.into());
        self
    }

    pub fn data(mut self, data: impl Into<Cow<'static, str>>) -> Self {
        self.data = Some(data.into());
        self
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(comment) = &self.comment {
            writeln!(f, ": {comment}")?;
        }
        if let Some(retry) = &self.retry {
            writeln!(f, "retry: {}", retry.as_millis())?;
        }
        if let Some(id) = &self.id {
            writeln!(f, "id: {id}")?;
        }
        if let Some(event) = &self.event {
            writeln!(f, "event: {event}")?;
        }
        if let Some(data) = &self.data {
            for line in data.lines() {
                writeln!(f, "data: {line}")?;
            }
        }
        writeln!(f)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Event;

    #[test]
    fn comment() {
        let event = Event::new().comment("xx");
        assert_eq!(format!("{event}"), ": xx\n\n");
    }

    #[test]
    fn retry() {
        let event = Event::new().retry(Duration::from_secs(1));
        assert_eq!(format!("{event}"), "retry: 1000\n\n");
    }

    #[test]
    fn id() {
        let event = Event::new().id("1");
        assert_eq!(format!("{event}"), "id: 1\n\n");
    }

    #[test]
    fn event() {
        let event = Event::new().event("message");
        assert_eq!(format!("{event}"), "event: message\n\n");
    }

    #[test]
    fn data() {
        let event = Event::new().data("hello\nworld\n");
        assert_eq!(format!("{event}"), "data: hello\ndata: world\n\n");
    }

    #[test]
    fn all() {
        let event = Event::new()
            .comment("xx")
            .retry(Duration::from_secs(1))
            .id("1")
            .event("message")
            .data("hello\nworld\n");
        assert_eq!(
            format!("{event}"),
            ": xx\nretry: 1000\nid: 1\nevent: message\ndata: hello\ndata: world\n\n"
        );
    }
}
