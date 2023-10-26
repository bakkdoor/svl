use iced::{
    widget::{Column, Container, PickList, Scrollable, Text, TextInput},
    Application, Command, Element, Theme,
};

use svl_core::{db::DBConnection, text};

use crate::{
    errors::SearchError,
    message::Message,
    query,
    search::{SearchKind, SearchResult, SearchState},
};

pub struct App {
    current_search_kind: SearchKind,
    author_search: SearchState<text::Author>,
    text_search: SearchState<text::Text>,
    word_search: SearchState<text::Word>,
    db: svl_core::db::DBConnection,
}

pub struct Args {
    pub db: DBConnection,
}

impl App {
    fn new(args: Args) -> Self {
        Self {
            db: args.db,
            current_search_kind: SearchKind::default(),
            author_search: SearchState::default(),
            text_search: SearchState::default(),
            word_search: SearchState::default(),
        }
    }

    fn view_search_kind(&self) -> Element<Message> {
        match self.current_search_kind {
            SearchKind::Author => self.view_authors(),
            SearchKind::Text => self.view_texts(),
            SearchKind::Word => self.view_words(),
        }
    }

    fn view_words(&self) -> Element<Message> {
        // list all words from search results
        self.word_search
            .search_results
            .iter()
            .fold(Column::new(), |col, word| {
                col.push(Text::new(word.to_string()))
            })
            .into()
    }

    fn view_texts(&self) -> Element<Message> {
        // list all texts from search results
        self.text_search
            .search_results
            .iter()
            .fold(Column::new(), |col, text| col.push(Text::new(&text.url)))
            .into()
    }

    fn view_authors(&self) -> Element<Message> {
        // list all authors from search results
        self.author_search
            .search_results
            .iter()
            .fold(Column::new(), |col, author| {
                col.push(Text::new(&author.name))
            })
            .into()
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
        match self.current_search_kind {
            SearchKind::Author => {
                let term = self.author_search.search_term();
                let db = self.db.clone();
                let task = query::search_authors(db, term);
                Command::perform(task, Message::SearchCompleted)
            }
            SearchKind::Text => {
                let term = self.word_search.search_term();
                let db = self.db.clone();
                let task = query::search_texts(db, term);
                Command::perform(task, Message::SearchCompleted)
            }
            SearchKind::Word => {
                let term = self.word_search.search_term();
                let db = self.db.clone();
                let task = query::search_words(db, term);
                Command::perform(task, Message::SearchCompleted)
            }
        }
    }

    fn update_search_results(&mut self, result: SearchResult) -> Result<(), SearchError> {
        match result {
            Ok(rows) => {
                match rows.kind() {
                    SearchKind::Author => {
                        self.author_search.update_search_results(rows.try_into()?);
                    }
                    SearchKind::Text => {
                        self.text_search.update_search_results(rows.try_into()?);
                    }
                    SearchKind::Word => {
                        self.word_search.update_search_results(rows.try_into()?);
                    }
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Theme = Theme;
    type Message = Message;
    type Flags = Args;

    fn new(args: Args) -> (Self, Command<Message>) {
        (Self::new(args), Command::none())
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
                println!("Search for: {}", self.search_term());

                self.search_command()
            }
            Message::SearchKindChanged(kind) => {
                self.current_search_kind = kind;
                Command::none()
            }
            Message::SearchCompleted(result) => {
                match self.update_search_results(result) {
                    Ok(_) => println!("Search completed successfully"),
                    Err(err) => println!("Search failed: {}", err),
                }
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let search_term: String = self.search_term();

        let result_counter = Text::new(format!(
            "Found {} results",
            match self.current_search_kind {
                SearchKind::Author => self.author_search.search_results.len(),
                SearchKind::Text => self.text_search.search_results.len(),
                SearchKind::Word => self.word_search.search_results.len(),
            }
        ));

        let side_padding = iced::Padding {
            left: 10.0,
            right: 10.0,
            top: 0.0,
            bottom: 0.0,
        };
        let fill = iced::Length::Fill;

        let input = TextInput::new("Search...", &search_term)
            .on_input(Message::InputChanged)
            .on_submit(Message::Search)
            .padding(10);

        let pick_list = PickList::new(
            vec![SearchKind::Word, SearchKind::Author, SearchKind::Text],
            Some(self.current_search_kind),
            Message::SearchKindChanged,
        );

        Container::new(
            Column::new()
                .push(padded_container(pick_list))
                .push(padded_container(result_counter).padding(side_padding))
                .push(padded_container(input.padding(10)).width(fill))
                .push(Scrollable::new(
                    Container::new(self.view_search_kind())
                        .width(fill)
                        .height(fill)
                        .padding(side_padding),
                )),
        )
        .width(fill)
        .height(fill)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn padded_container<'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    Container::new(content).padding(10)
}
