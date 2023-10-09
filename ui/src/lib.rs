use iced::{
    widget::{Column, Container, PickList, Text, TextInput},
    Application, Command, Element, Settings, Theme,
};
use svl_core::{
    db::{DBConnection, DBError, DBParams, NamedRows},
    text,
};
use thiserror::Error;

pub fn run_ui() -> iced::Result {
    SearchApp::run(Settings::default())
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    Closed,
    InputChanged(String),
    Search,
    SearchKindChanged(SearchKind),
    SearchCompleted(SearchResult),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchKind {
    Author,
    Text,
    #[default]
    Word,
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

type SearchResult = Result<SearchRows, SearchError>;

#[derive(Debug, Clone, Error)]
pub enum SearchError {
    #[error("DBError: {0}")]
    Db(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<DBError> for SearchError {
    fn from(err: DBError) -> Self {
        SearchError::Db(err.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct SearchRows {
    pub kind: SearchKind,
    pub rows: NamedRows,
}

impl SearchRows {
    pub fn new(kind: SearchKind, rows: NamedRows) -> Self {
        Self { kind, rows }
    }

    pub fn rows(&self) -> &NamedRows {
        &self.rows
    }

    pub fn kind(&self) -> SearchKind {
        self.kind
    }
}

#[derive(Default)]
pub struct SearchApp {
    db: DBConnection,
    current_search_kind: SearchKind,
    author_search: SearchState<text::Author>,
    text_search: SearchState<text::Text>,
    word_search: SearchState<text::Word>,
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

    fn search_command(&self) -> Command<Message> {
        // let term = self.author_search.search_term();

        match self.current_search_kind {
            SearchKind::Author => {
                // let task = search_authors(&self.db, &term);
                // Command::perform(task, Message::SearchCompleted)
                Command::none()
            }
            SearchKind::Text => {
                // Command::perform(self.db.search_texts(&term), Message::SearchCompleted)
                Command::none()
            }
            SearchKind::Word => {
                // Command::perform(self.db.search_words(&term), Message::SearchCompleted)
                Command::none()
            }
        }
    }
}

impl Application for SearchApp {
    type Executor = iced::executor::Default;
    type Theme = Theme;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Search App")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::InputChanged(term) => {
                self.update_search(&term);
                Command::none()
            }
            Message::Search => {
                // Implement the actual search logic here based on self.search_term

                self.search_command()
            }
            Message::SearchKindChanged(kind) => {
                self.current_search_kind = kind;
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let search_term: String = self.search_term();
        let input = TextInput::new("Search...", &search_term).on_input(Message::InputChanged);

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

#[allow(dead_code)]
async fn search_authors(db: &DBConnection, term: &str) -> SearchResult {
    let script = r#"
    ?[author_id,name,url] :=
        *Author { name, url },
        str_include(name, $name)
    "#;
    let params = DBParams::from_iter(vec![("name".into(), term.into())]);
    let rows = db.run_immutable(script, params).await?;
    Ok(SearchRows::new(SearchKind::Author, rows))
}
