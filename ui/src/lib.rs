use iced::{widget::text, Element, Sandbox, Settings, Theme};
use svl_core::db::DBConnection;

pub fn run_ui(_db: &DBConnection) -> iced::Result {
    App::run(Settings::default())
}

#[derive(Default)]
struct App {}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    Search(SearchTerm),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum SearchTerm {
    Word(String),
    Author(String),
    Text(String),
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        App {}
    }

    fn title(&self) -> String {
        String::from("Statistica Verbōrum Latīna")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Search(SearchTerm::Word(word)) => {
                println!("Word search: {}", word);
            }
            Message::Search(SearchTerm::Author(author)) => {
                println!("Author search: {}", author);
            }
            Message::Search(SearchTerm::Text(text)) => {
                println!("Text search: {}", text);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        text("Hello, world!").into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
