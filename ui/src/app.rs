use iced::{
    widget::{Column, Container, PickList, Row, Scrollable, Text, TextInput},
    Application, Command, Element, Theme,
};

use svl_core::{db::DBConnection, text};

use crate::{
    errors::SearchError,
    message::Message,
    query,
    search::{SearchKind, SearchMode, SearchResult, SearchState},
};

pub struct App {
    current_search_kind: SearchKind,
    current_search_mode: SearchMode,
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
            current_search_mode: SearchMode::default(),
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

    fn search_kind(&self) -> SearchKind {
        self.current_search_kind
    }

    fn search_mode(&self) -> SearchMode {
        self.current_search_mode
    }

    fn update_search(&mut self, term: &str) {
        match self.current_search_kind {
            SearchKind::Author => self.author_search.update_search(term),
            SearchKind::Text => self.text_search.update_search(term),
            SearchKind::Word => self.word_search.update_search(term),
        }
    }

    fn update_case_sensitive(&mut self, is_case_sensitive: bool) {
        match self.current_search_kind {
            SearchKind::Author => self.author_search.update_case_sensitive(is_case_sensitive),
            SearchKind::Text => self.text_search.update_case_sensitive(is_case_sensitive),
            SearchKind::Word => self.word_search.update_case_sensitive(is_case_sensitive),
        }
    }

    fn is_case_sensitive(&self) -> bool {
        match self.current_search_kind {
            SearchKind::Author => self.author_search.is_case_sensitive,
            SearchKind::Text => self.text_search.is_case_sensitive,
            SearchKind::Word => self.word_search.is_case_sensitive,
        }
    }

    fn search_command(&self) -> Command<Message> {
        let search_mode = self.current_search_mode;
        match self.current_search_kind {
            SearchKind::Author => {
                let term = self.author_search.search_term();
                let db = self.db.clone();
                let task = query::search_authors(db, term, search_mode);
                Command::perform(task, Message::SearchCompleted)
            }
            SearchKind::Text => {
                let term = self.word_search.search_term();
                let db = self.db.clone();
                let task = query::search_texts(db, term, search_mode);
                Command::perform(task, Message::SearchCompleted)
            }
            SearchKind::Word => {
                let term = self.word_search.search_term();
                let db = self.db.clone();
                let task = query::search_words(db, term, search_mode);
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
            Message::Closed => Command::none(),

            Message::InputChanged(term) => {
                self.update_search(&term);
                Command::none()
            }
            Message::Search => {
                // Implement the actual search logic here based on self.search_term
                println!(
                    "Search {} {}: {}",
                    self.search_kind(),
                    self.search_mode(),
                    self.search_term()
                );

                self.search_command()
            }
            Message::SearchKindChanged(kind) => {
                self.current_search_kind = kind;
                Command::none()
            }
            Message::SearchModeChanged(mode) => {
                self.current_search_mode = mode;
                Command::none()
            }
            Message::SearchCompleted(result) => {
                match self.update_search_results(result) {
                    Ok(_) => println!("Search completed successfully"),
                    Err(err) => println!("Search failed: {}", err),
                }
                Command::none()
            }
            Message::CaseSensitiveChanged(is_case_sensitive) => {
                self.update_case_sensitive(is_case_sensitive);
                Command::none()
            }
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

        let search_kind_pick_list = PickList::new(
            SearchKind::all_kinds(),
            Some(self.current_search_kind),
            Message::SearchKindChanged,
        );

        let search_mode_pick_list = PickList::new(
            SearchMode::all_modes(),
            Some(self.current_search_mode),
            Message::SearchModeChanged,
        );

        // checkbox for case sensitive search
        let case_sensitive_checkbox = iced::widget::checkbox::Checkbox::new(
            "Case sensitive",
            self.is_case_sensitive(),
            Message::CaseSensitiveChanged,
        );

        let picklist_row = Row::new()
            .spacing(10)
            .push(search_kind_pick_list)
            .push(search_mode_pick_list)
            .push(case_sensitive_checkbox);

        Container::new(
            Column::new()
                .push(padded_container(picklist_row))
                .push(padded_container(result_counter).padding(side_padding))
                .push(padded_container(input.padding(10)).width(fill))
                .push(Scrollable::new(
                    padded_container(self.view_search_kind()).width(fill),
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
