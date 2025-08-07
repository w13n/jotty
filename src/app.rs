use std::collections::HashMap;
use time::Date;

struct App {
    entries: HashMap<Date, Entry>,
}

struct Entry {
    events: Vec<Event>,
    tasks: Vec<Task>,
}

struct Event {
    title: String,
    importance: Importance,
}

enum Importance {
    Low,
    Normal,
    High,
    Extreme,
}

struct Task {
    title: String,
    completion_level: CompletionLevel,
}

enum CompletionLevel {
    None,
    Partial,
    Full,
}
