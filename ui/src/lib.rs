use iced::{
    widget::{Column, Container, PickList, Text, TextInput},
    Element, Sandbox, Settings, Theme,
};
use svl_core::{db::DBConnection, text};

pub fn run_ui(_db: &DBConnection) -> iced::Result {
    SearchApp::run(Settings::default())
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    Closed,
    InputChanged(String),
    OptionHovered(SearchKind),
    Search,
    SearchKindChanged(SearchKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchKind {
    Author,
    Text,
    Word,
}

impl Default for SearchKind {
    fn default() -> Self {
        SearchKind::Word
    }
}

impl std::fmt::Display for SearchKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchKind::Author => write!(f, "Authors"),
            SearchKind::Text => write!(f, "Texts"),
            SearchKind::Word => write!(f, "Words"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchState<Result> {
    pub search_term: String,
    pub search_results: Vec<Result>,
}

impl<Result> SearchState<Result> {
    pub fn update_search(&mut self, term: &str) {
        self.search_term = term.to_string();
        self.search_results = Vec::new();
    }

    pub fn search_term(&self) -> String {
        self.search_term.clone()
    }
}

impl<Result> Default for SearchState<Result> {
    fn default() -> Self {
        Self {
            search_term: String::new(),
            search_results: Vec::new(),
        }
    }
}

pub struct SearchApp {
    current_search_kind: SearchKind,
    author_search: SearchState<text::Author>,
    text_search: SearchState<text::Text>,
    word_search: SearchState<text::Word>,
}

impl Default for SearchApp {
    fn default() -> Self {
        Self {
            current_search_kind: SearchKind::default(),
            author_search: SearchState::default(),
            text_search: SearchState::default(),
            word_search: SearchState::default(),
        }
    }
}

impl SearchApp {
    fn view_search_kind(&self) -> Element<Message> {
        match self.current_search_kind {
            SearchKind::Author => self.view_authors(),
            SearchKind::Text => self.view_texts(),
            SearchKind::Word => self.view_words(),
        }
    }

    fn view_words(&self) -> Element<Message> {
        // Implement the view for the Words search
        // ...
        Text::new("Words Search").into()
    }

    fn view_texts(&self) -> Element<Message> {
        // Implement the view for the Texts search
        // ...
        Text::new("Texts Search").into()
    }

    fn view_authors(&self) -> Element<Message> {
        // Implement the view for the Authors search
        // ...
        Text::new("Authors Search").into()
    }

    fn search_term(&self) -> String {
        match self.current_search_kind {
            SearchKind::Author => self.author_search.search_term(),
            SearchKind::Text => self.text_search.search_term(),
            SearchKind::Word => self.word_search.search_term(),
        }
    }

    fn update_search(&mut self, term: &str) {
        match self.current_search_kind {
            SearchKind::Author => self.author_search.update_search(term),
            SearchKind::Text => self.text_search.update_search(term),
            SearchKind::Word => self.word_search.update_search(term),
        }
    }
}

impl Sandbox for SearchApp {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Search App")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::InputChanged(term) => {
                self.update_search(&term);
            }
            Message::Search => {
                // Implement the actual search logic here based on self.search_term
            }
            Message::SearchKindChanged(kind) => {
                self.current_search_kind = kind;
            }
            _ => {}
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let search_term: String = self.search_term();
        let input =
            TextInput::new("Search...", &search_term).on_input(|term| Message::InputChanged(term));

        let pick_list = PickList::new(
            vec![SearchKind::Word, SearchKind::Author, SearchKind::Text],
            Some(self.current_search_kind),
            Message::SearchKindChanged,
        );

        Container::new(
            Column::new()
                .push(pick_list)
                .push(input)
                .push(self.view_search_kind()),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
