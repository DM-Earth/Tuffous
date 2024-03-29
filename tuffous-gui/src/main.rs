mod appearance;
mod config;

use chrono::{Datelike, Local};
use iced::{
    alignment, executor, theme,
    widget::{
        button, column, container, horizontal_space, row, scrollable, text, text_input,
        vertical_space,
    },
    window, Application, Color, Element, Length, Renderer, Settings, Theme,
};
use tuffous_core::{util, Todo, TodoInstance};

struct App {
    pub instance: TodoInstance,
    pub states: Vec<TodoState>,
    pub dep_selection: Option<(u64, Vec<u64>)>,
    pub range: Vec<u64>,
    pub complete_filter: TodoCompleteFilter,
    pub view: TodoView,
    pub search_cache: String,
    pub search: bool,
    pub config: config::ConfigInstance,
}

fn main() -> iced::Result {
    App::run(Settings {
        window: window::Settings {
            size: (850, 700),
            min_size: Some((600, 650)),
            icon: Some(window::icon::from_file_data(include_bytes!("../icon.png"), None).unwrap()),
            ..window::Settings::default()
        },
        default_font: appearance::FONT.as_ref().copied().unwrap_or_default(),
        ..Settings::default()
    })
}

#[derive(PartialEq, Eq)]
enum TodoCompleteFilter {
    Completed,
    NotComplete,
    All,
}

impl TodoCompleteFilter {
    pub fn test(&self, todo: &Todo) -> bool {
        match self {
            Self::Completed => todo.completed,
            Self::NotComplete => !todo.completed,
            Self::All => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TodoView {
    Today,
    Upcoming,
    Anytime,
    Logbook,
    All,
    Project(u64),
}

impl TodoView {
    pub fn title(&self, instance: &TodoInstance, theme: Theme) -> (char, String, Color) {
        let style = || appearance::StyleSheet::from_theme(&theme);
        match self {
            TodoView::Today => ('', String::from("Today"), style().star),
            TodoView::Upcoming => ('󰸗', String::from("Upcoming"), style().flag),
            TodoView::Anytime => ('', String::from("Anytime"), style().blue_green),
            TodoView::Logbook => ('󱓵', String::from("Logbook"), style().green),
            TodoView::All => ('󰾍', String::from("All"), style().gray),
            TodoView::Project(id) => (
                completion_state_view(*id, instance),
                instance.get(*id).unwrap().metadata.name.to_owned(),
                style().gray,
            ),
        }
    }

    pub fn filter(
        &self,
        instance: &TodoInstance,
        complete: &TodoCompleteFilter,
        keywords: &Vec<String>,
    ) -> Vec<u64> {
        let get_relatives = |id: u64| {
            let mut vec = Vec::new();
            for father in instance.all_deps(id) {
                if !vec.contains(&father) {
                    vec.push(father)
                }
            }

            vec
        };

        let mut vec = Vec::new();

        for todo in &instance.todos {
            if complete.test(todo)
                && self.test(todo.id(), instance)
                && (keywords.is_empty() || {
                    let mut b = true;
                    for key in keywords {
                        b = b
                            && (todo.metadata.name.to_lowercase().contains(key)
                                || todo.metadata.details.to_lowercase().contains(key)
                                || {
                                    let mut c = false;
                                    for tag in &todo.tags {
                                        if tag.to_lowercase().contains(key) {
                                            c = true;
                                            break;
                                        }
                                    }
                                    c
                                })
                    }
                    b
                })
            {
                if !vec.contains(&todo.id()) {
                    vec.push(todo.id());
                }

                for father in get_relatives(todo.id()) {
                    if !vec.contains(&father) {
                        vec.push(father);
                    }
                }

                for child in instance.children(todo.id()) {
                    if complete.test(instance.get(child).unwrap()) && !vec.contains(&child) {
                        vec.push(child);
                    }
                }
            }

            if vec.len() > 1024 {
                break;
            }
        }

        vec
    }

    pub fn test(&self, id: u64, instance: &TodoInstance) -> bool {
        let todo = instance.get(id).unwrap();
        match self {
            TodoView::Today => {
                if let Some(date) = todo.time {
                    date.eq(&Local::now().date_naive())
                } else if let Some(ddl) = todo.deadline {
                    ddl.date() <= Local::now().date_naive()
                } else {
                    false
                }
            }
            TodoView::Upcoming => {
                if let Some(date) = todo.time {
                    !date.eq(&Local::now().date_naive())
                } else if let Some(ddl) = todo.deadline {
                    Local::now().date_naive() < ddl.date()
                } else {
                    false
                }
            }
            TodoView::Anytime => todo.time.is_none() && todo.deadline.is_none(),
            TodoView::Logbook => todo.completed,
            TodoView::All => true,
            TodoView::Project(project_id) => {
                instance.children(*project_id).contains(&id) || &id == project_id
            }
        }
    }

    pub fn default_complete_filter(&self) -> TodoCompleteFilter {
        match self {
            Self::Logbook => TodoCompleteFilter::Completed,
            _ => TodoCompleteFilter::NotComplete,
        }
    }

    pub fn process_todo(&self, todo: &mut Todo) {
        if !self.allow_create_todo() {
            unreachable!()
        }

        match self {
            Self::Today => {
                todo.time = Some(Local::now().date_naive());
            }
            Self::Project(project) => {
                todo.dependents.push(*project);
            }
            _ => (),
        }
    }

    pub fn allow_create_todo(&self) -> bool {
        matches!(
            self,
            Self::Today | Self::Project(_) | Self::Anytime | Self::All
        )
    }
}

impl App {
    pub fn state(&self, id: u64) -> Option<&TodoState> {
        self.states.iter().find(|&state| state.id == id)
    }

    pub fn state_mut(&mut self, id: u64) -> Option<&mut TodoState> {
        self.states.iter_mut().find(|state| state.id == id)
    }

    pub fn refresh_states(&mut self) {
        util::remove_from_vec_if(&mut self.states, &|state| {
            !self.instance.todos().contains(&state.id)
        });

        for todo in &self.instance.todos {
            if util::vec_none_match(&self.states, &|state| state.id == todo.id()) {
                self.states.push(TodoState::new(todo));
            }
        }
    }

    pub fn style_sheet(&self) -> appearance::StyleSheet {
        appearance::StyleSheet::from_theme(&self.theme())
    }

    fn view_todos(&self) -> iced::Element<Message> {
        container(if self.range.is_empty() && !self.search {
            container(
                appearance::icon(self.view.title(&self.instance, self.theme()).0)
                    .style(theme::Text::Color(self.style_sheet().gray))
                    .size(80)
                    .width(Length::Fill),
            )
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
        } else {
            let mut todos = Vec::new();
            for todo_id in self.instance.todos() {
                todos.push((
                    self.instance.get(todo_id).unwrap(),
                    self.state(todo_id).unwrap(),
                ));
            }

            let todo_views = scrollable(
                column({
                    let mut vec: Vec<Element<'_, Message, Renderer>> = Vec::new();

                    if self.search {
                        vec.push(horizontal_space(35).into());
                        vec.push(
                            container(
                                text_input("Search", &self.search_cache)
                                    .on_input(|s| Message::CacheSearchContent(s))
                                    .width(360),
                            )
                            .center_x()
                            .width(Length::Fill)
                            .into(),
                        );
                    }

                    vec.push(horizontal_space(35).into());

                    for todo in &self.instance.todos {
                        if todo.dependents.is_empty() && self.range.contains(&todo.id()) {
                            for view in &mut self.state(todo.id()).unwrap().view(self) {
                                let mut row_c: Vec<Element<'_, Message, Renderer>> = Vec::new();
                                row_c.push(horizontal_space(view.0).into());
                                row_c.append(&mut view.1);
                                vec.push(
                                    container(container(row(row_c)).max_width(1500))
                                        .align_x(alignment::Horizontal::Center)
                                        .width(Length::Fill)
                                        .into(),
                                );
                            }
                        }
                    }
                    vec
                })
                .spacing(7.5),
            );

            container(todo_views).center_x()
        })
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }

    fn view_sidebar(&self) -> iced::Element<Message> {
        let height = 30;
        let mut self_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();

        self_vec.push(vertical_space(7.5).into());

        let view_button = |view: TodoView| -> Element<'_, Message, Renderer> {
            row!(
                horizontal_space(7.5),
                container(
                    button(row!(
                        appearance::icon(view.title(&self.instance, self.theme()).0).style(
                            theme::Text::Color(view.title(&self.instance, self.theme()).2)
                        ),
                        text(format!("  {}", view.title(&self.instance, self.theme()).1)).size(15),
                        horizontal_space(Length::Fill)
                    ))
                    .on_press(Message::SwitchView(view.clone()))
                    .style(theme::Button::Text),
                )
                .style(if self.view.eq(&view) {
                    theme::Container::Box
                } else {
                    theme::Container::Transparent
                })
                .height(height)
                .center_y()
                .align_x(alignment::Horizontal::Left)
                .width(Length::Fill)
                .max_width(200)
            )
            .into()
        };

        self_vec.push(view_button(TodoView::Today));
        self_vec.push(view_button(TodoView::Upcoming));
        self_vec.push(view_button(TodoView::Anytime));
        self_vec.push(view_button(TodoView::Logbook));
        self_vec.push(view_button(TodoView::All));

        {
            let mut pinned = Vec::new();
            for todo in &self.instance.todos {
                for tag in &todo.tags {
                    if !todo.completed && tag.to_lowercase().eq("pinned") {
                        pinned.push(todo.id());
                        break;
                    }
                }
            }

            if !pinned.is_empty() {
                self_vec.push(vertical_space(15).into());
                for pin in pinned {
                    self_vec.push(view_button(TodoView::Project(pin)));
                }
            }
        }

        self_vec.push(vertical_space(Length::Fill).into());

        let mut controls_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();
        controls_vec.push(horizontal_space(7.5).into());

        controls_vec.push(
            container(
                button(appearance::icon('󰔎').style(theme::Text::Color(self.style_sheet().gray)))
                    .style(theme::Button::Text)
                    .on_press(Message::UpdateConfig(ConfigMessage::ToggleDarkMode)),
            )
            .height(height)
            .center_y()
            .into(),
        );

        self_vec.push(
            container(row(controls_vec).height(height))
                .align_x(alignment::Horizontal::Left)
                .into(),
        );

        self_vec.push(vertical_space(7.5).into());

        container(column(self_vec))
            .width(200)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Left)
            .into()
    }

    pub fn view_controls(&self) -> iced::Element<Message> {
        let mut self_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();
        let height = 45;
        self_vec.push(horizontal_space(Length::FillPortion(1)).into());

        if self.view.allow_create_todo() {
            self_vec.push(
                container(
                    button(
                        appearance::icon('󰐕')
                            .style(theme::Text::Color(self.style_sheet().gray))
                            .size(25)
                            .width(Length::FillPortion(2)),
                    )
                    .style(theme::Button::Text)
                    .on_press(Message::CreateTodo),
                )
                .height(height)
                .center_y()
                .into(),
            );
            self_vec.push(horizontal_space(Length::FillPortion(2)).into());
        }

        if self
            .view
            .default_complete_filter()
            .eq(&TodoCompleteFilter::NotComplete)
        {
            self_vec.push(
                container(
                    button(
                        appearance::icon(if self.complete_filter.eq(&TodoCompleteFilter::All) {
                            '󰘽'
                        } else {
                            '󰘾'
                        })
                        .style(theme::Text::Color(self.style_sheet().gray))
                        .size(25)
                        .width(Length::FillPortion(2)),
                    )
                    .style(theme::Button::Text)
                    .on_press(Message::SwitchCompleteFilter),
                )
                .height(height)
                .center_y()
                .into(),
            );
            self_vec.push(horizontal_space(Length::FillPortion(2)).into());
        }

        self_vec.push(
            container(
                button(
                    appearance::icon(if self.search { '󰦀' } else { '󰍉' })
                        .style(theme::Text::Color(self.style_sheet().gray))
                        .size(25)
                        .width(Length::FillPortion(2)),
                )
                .style(theme::Button::Text)
                .on_press(Message::ToggleSearch),
            )
            .height(height)
            .center_y()
            .into(),
        );

        self_vec.push(horizontal_space(Length::FillPortion(1)).into());
        container(row(self_vec))
            .height(height)
            .center_y()
            .center_x()
            .into()
    }

    pub fn refresh_range(&mut self) {
        self.range = self.view.filter(
            &self.instance,
            &self.complete_filter,
            &if self.search {
                self.search_cache
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .collect()
            } else {
                Vec::new()
            },
        );
    }
}

impl Application for App {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let config = config::ConfigInstance::get();

        let mut app = App {
            instance: TodoInstance::create(&flags.path),
            states: Vec::new(),
            dep_selection: None,
            range: Vec::new(),
            complete_filter: TodoCompleteFilter::NotComplete,
            view: TodoView::Today,
            search_cache: String::new(),
            search: false,
            config,
        };

        app.instance.read_all();
        app.instance.refresh();
        app.refresh_states();
        app.refresh_range();
        app.config.write();

        (
            app,
            iced::font::load(include_bytes!("../fonts/nerd_font.ttf").as_slice())
                .map(|e| Message::LoadFont(e)),
        )
    }

    fn title(&self) -> String {
        String::from("Tuffous")
    }

    fn theme(&self) -> Self::Theme {
        if self.config.dark_theme {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        self.instance.refresh();
        match message {
            Message::TodoMessage(id, msg) => match msg {
                TodoMessage::ToggleComplete => {
                    let todo = self.instance.get_mut(id).unwrap();
                    todo.completed = !todo.completed;
                    self.refresh_range();
                }
                TodoMessage::Edit(edit_msg) => match edit_msg {
                    EditMessage::Name(name) => {
                        self.instance.get_mut(id).unwrap().metadata.name = name
                    }
                    EditMessage::Details(details) => {
                        self.instance.get_mut(id).unwrap().metadata.details = details
                    }
                    EditMessage::ToggleEdit => {
                        {
                            let todo = self.instance.get(id).unwrap();
                            let time_o = todo.time;
                            let ddl_o = todo.deadline;
                            let state = self.state_mut(id).unwrap();
                            state.editing = !state.editing;
                            if state.editing {
                                if let Some(time) = time_o {
                                    state.time_cache = time.format("%Y/%m/%d").to_string();
                                }

                                if let Some(ddl) = ddl_o {
                                    state.ddl_cache = ddl.format("%Y/%m/%d-%H:%M:%S").to_string();
                                }
                            } else {
                                state.time_cache = String::new();
                                state.ddl_cache = String::new();
                                self.dep_selection = None;
                            }
                        }
                        if self.state(id).unwrap().editing {
                            for state in &mut self.states {
                                if !state.id == id {
                                    state.editing = false;
                                }
                            }
                        }

                        {
                            if !self.state(id).unwrap().editing {
                                let todo = self.instance.get_mut(id).unwrap();
                                if todo.metadata.name.is_empty() {
                                    todo.metadata.name = String::from("untitled todo");
                                }
                            }
                        }

                        self.refresh_range();
                    }
                    EditMessage::Date(date) => {
                        if let Some(date_r) = util::parse_date(&date) {
                            self.instance.get_mut(id).unwrap().time = Some(date_r);
                        } else {
                            self.instance.get_mut(id).unwrap().time = None;
                        }
                        self.state_mut(id).unwrap().time_cache = date;
                    }
                    EditMessage::Deadline(ddl) => {
                        if let Some(ddl_r) = util::parse_date_and_time(&ddl) {
                            self.instance.get_mut(id).unwrap().deadline = Some(ddl_r);
                        } else {
                            self.instance.get_mut(id).unwrap().deadline = None;
                        }
                        self.state_mut(id).unwrap().ddl_cache = ddl;
                    }
                    EditMessage::Tags(tags) => {
                        let todo = self.instance.get_mut(id).unwrap();
                        todo.tags.clear();
                        for tag in tags.split_whitespace() {
                            todo.tags.push(tag.to_string());
                        }
                    }
                    EditMessage::ToggleSelectChildren => {
                        if self.dep_selection.is_some() {
                            self.dep_selection = None;
                        } else {
                            self.dep_selection = Some((id, self.instance.children_once(id)));
                        }
                    }
                },
                TodoMessage::ExpandToggle => {
                    let state = self.state_mut(id).unwrap();
                    state.expanded = !state.expanded;
                }
                TodoMessage::Delete => {
                    match self.view {
                        TodoView::Project(idd) => {
                            if idd == id {
                                self.view = TodoView::Today;
                            }
                        }
                        _ => (),
                    }
                    self.instance.remove(id);
                    self.refresh_states();
                    self.refresh_range();
                }
                TodoMessage::ToggleChild => {
                    if let Some((father_id, child_vec)) = &mut self.dep_selection {
                        if child_vec.contains(&id) {
                            util::remove_from_vec(
                                &mut self.instance.get_mut(id).unwrap().dependents,
                                father_id,
                            );
                            util::remove_from_vec(child_vec, &id);
                        } else {
                            self.instance.child(*father_id, id);
                            child_vec.push(id);
                        }
                    }
                }
            },
            Message::SwitchView(view) => {
                self.view = view;
                self.complete_filter = self.view.default_complete_filter();
                self.refresh_range();
                self.states.clear();
                self.refresh_states();
            }
            Message::CreateTodo => {
                let mut todo = Todo::create(String::from("untitled todo"));
                self.view.process_todo(&mut todo);
                let id = todo.id();
                if !self.instance.todos().contains(&id) {
                    self.instance.todos.push(todo);
                }
                self.refresh_states();
                self.refresh_range();

                let destroy = |_command: iced::Command<Message>| {};
                destroy(self.update(Message::TodoMessage(
                    id,
                    TodoMessage::Edit(EditMessage::ToggleEdit),
                )));
            }
            Message::SwitchCompleteFilter => {
                if self
                    .complete_filter
                    .eq(&self.view.default_complete_filter())
                {
                    self.complete_filter = TodoCompleteFilter::All
                } else {
                    self.complete_filter = self.view.default_complete_filter()
                }
                self.refresh_range();
            }
            Message::ToggleSearch => {
                self.search = !self.search;
                if !self.search {
                    self.search_cache = String::new();
                }
            }
            Message::CacheSearchContent(string) => {
                self.search_cache = string;
                self.refresh_range();
            }
            Message::UpdateConfig(msg) => {
                match msg {
                    ConfigMessage::ToggleDarkMode => {
                        self.config.dark_theme = !self.config.dark_theme;
                    }
                }
                self.config.write();
            }
            _ => (),
        };

        self.instance.write_all();
        self.refresh_states();
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        row(vec![
            self.view_sidebar(),
            column(vec![self.view_todos(), self.view_controls()])
                .width(Length::Fill)
                .into(),
        ])
        .height(Length::Fill)
        .into()
    }
}

struct Flags {
    pub path: String,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            path: String::from("."),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    TodoMessage(u64, TodoMessage),
    SwitchView(TodoView),
    CreateTodo,
    SwitchCompleteFilter,
    ToggleSearch,
    CacheSearchContent(String),
    UpdateConfig(ConfigMessage),
    LoadFont(Result<(), iced::font::Error>),
}

#[derive(Debug, Clone)]
enum ConfigMessage {
    ToggleDarkMode,
}

#[derive(Debug, Clone)]
enum TodoMessage {
    ToggleComplete,
    Edit(EditMessage),
    ExpandToggle,
    Delete,
    ToggleChild,
}

#[derive(Debug, Clone)]
enum EditMessage {
    Name(String),
    Details(String),
    Date(String),
    Deadline(String),
    Tags(String),
    ToggleEdit,
    ToggleSelectChildren,
}

struct TodoState {
    pub id: u64,
    pub editing: bool,
    pub expanded: bool,
    pub time_cache: String,
    pub ddl_cache: String,
}

impl TodoState {
    pub fn new(todo: &Todo) -> Self {
        TodoState {
            id: todo.id(),
            editing: false,
            expanded: true,
            time_cache: String::new(),
            ddl_cache: String::new(),
        }
    }

    pub fn view<'a>(&'a self, app: &'a App) -> Vec<(u16, Vec<Element<'_, Message, Renderer>>)> {
        let height = 28.0;

        let todo = app.instance.get(self.id).unwrap();
        let mut self_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();

        if app.instance.children_once(self.id).is_empty() {
            self_vec.push(horizontal_space(25).into());
        } else {
            self_vec.push(
                container(
                    button(
                        appearance::icon(if self.expanded { '' } else { '' })
                            .style(theme::Text::Color(app.style_sheet().gray)),
                    )
                    .width(20)
                    .style(theme::Button::Text)
                    .on_press(Message::TodoMessage(
                        self.id.to_owned(),
                        TodoMessage::ExpandToggle,
                    )),
                )
                .height(height)
                .center_y()
                .into(),
            );

            self_vec.push(horizontal_space(5).into());
        }

        self_vec.push(
            container(
                button(
                    appearance::icon(completion_state_view(self.id, &app.instance))
                        .size(17)
                        .style(theme::Text::Color(
                            if app.instance.children_once(self.id).is_empty()
                                && !app.instance.get(self.id).unwrap().completed
                            {
                                app.style_sheet().gray
                            } else {
                                app.style_sheet().checkbox
                            },
                        )),
                )
                .on_press(Message::TodoMessage(
                    self.id.to_owned(),
                    TodoMessage::ToggleComplete,
                ))
                .style(theme::Button::Text),
            )
            .height(height)
            .center_y()
            .into(),
        );

        let mut left_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();
        let mut right_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();

        if !self.editing {
            // Todo information
            if let Some(time) = &todo.time {
                left_vec.push(
                    if time.eq(&Local::now().date_naive()) {
                        container(
                            appearance::icon('')
                                .size(15)
                                .style(theme::Text::Color(app.style_sheet().star))
                                .height(17.5),
                        )
                    } else {
                        container(
                            text(format!(
                                "  {}{} {}  ",
                                if time.year() == Local::now().year() {
                                    String::from("")
                                } else {
                                    format!("{} ", time.year())
                                },
                                util::month_str(time.month()),
                                time.day()
                            ))
                            .size(18.5),
                        )
                    }
                    .style(if time.eq(&Local::now().date_naive()) {
                        theme::Container::Transparent
                    } else {
                        theme::Container::Box
                    })
                    .height(height)
                    .center_y()
                    .into(),
                );

                left_vec.push(horizontal_space(3.5).into());
            }

            left_vec.push(
                container(
                    button(text(&todo.metadata.name).size(15))
                        .style(theme::Button::Text)
                        .on_press(Message::TodoMessage(
                            self.id,
                            TodoMessage::Edit(EditMessage::ToggleEdit),
                        ))
                        .height(Length::Fill),
                )
                .height(height)
                .center_y()
                .into(),
            );

            if !todo.tags.is_empty() {
                left_vec.push(horizontal_space(3.5).into());

                for tag in &todo.tags {
                    left_vec.push(horizontal_space(5).into());

                    left_vec.push(
                        container(
                            container(text(format!("   {tag}   ")).size(12.5))
                                .style(theme::Container::Custom(Box::new(appearance::TagStyle)))
                                .height(height - 7.0)
                                .center_y(),
                        )
                        .center_y()
                        .height(height)
                        .into(),
                    );

                    left_vec.push(horizontal_space(2.5).into());
                }
            }

            if let Some(ddl) = &todo.deadline {
                right_vec.push(
                    container(
                        text(format!(
                            "{} {} ",
                            if ddl.date().eq(&Local::now().date_naive()) {
                                String::from("Today")
                            } else {
                                format!(
                                    "{}{} {}",
                                    if ddl.year().eq(&Local::now().year()) {
                                        String::new()
                                    } else {
                                        format!("{} ", ddl.year())
                                    },
                                    util::month_str(ddl.month()),
                                    ddl.day()
                                )
                            },
                            ddl.time().format("%H:%M")
                        ))
                        .size(14)
                        .style(theme::Text::Color(app.style_sheet().flag)),
                    )
                    .height(height)
                    .center_y()
                    .into(),
                );

                right_vec.push(
                    container(
                        appearance::icon(if ddl > &Local::now().naive_local() {
                            '󰈻'
                        } else {
                            '󰮛'
                        })
                        .style(theme::Text::Color(app.style_sheet().flag)),
                    )
                    .height(height)
                    .center_y()
                    .into(),
                );
            }

            if let Some((father_id, child_vec)) = &app.dep_selection {
                if app.instance.is_child_able(*father_id, self.id) || child_vec.contains(&self.id) {
                    right_vec.push(horizontal_space(7.5).into());
                    right_vec.push(
                        container(
                            button(
                                appearance::icon(if !child_vec.contains(&self.id) {
                                    '󰝦'
                                } else {
                                    '󰻃'
                                })
                                .size(15),
                            )
                            .style(theme::Button::Text)
                            .on_press(Message::TodoMessage(self.id, TodoMessage::ToggleChild)),
                        )
                        .height(height)
                        .into(),
                    );
                }
            }
        } else {
            let mut col_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();

            // Todo Editing
            col_vec.push(
                row!(
                    container(appearance::icon('󰑕')).height(height).center_y(),
                    container(
                        text_input("Input title here", &todo.metadata.name)
                            .on_input(|input| {
                                Message::TodoMessage(
                                    self.id.to_owned(),
                                    TodoMessage::Edit(EditMessage::Name(input)),
                                )
                            })
                            .width(350)
                    )
                    .height(height)
                    .center_y()
                )
                .into(),
            );
            col_vec.push(
                row!(
                    container(appearance::icon('󰟃')).height(height).center_y(),
                    container(
                        text_input("Input details here", &todo.metadata.details)
                            .on_input(|input| {
                                Message::TodoMessage(
                                    self.id.to_owned(),
                                    TodoMessage::Edit(EditMessage::Details(input)),
                                )
                            })
                            .width(350)
                    )
                    .height(height)
                    .center_y()
                )
                .into(),
            );
            col_vec.push(
                row!(
                    container(appearance::icon('󰃯')).height(height).center_y(),
                    container(
                        text_input("Input date here", &self.time_cache)
                            .on_input(|input| {
                                Message::TodoMessage(
                                    self.id.to_owned(),
                                    TodoMessage::Edit(EditMessage::Date(input)),
                                )
                            })
                            .width(350)
                    )
                    .height(height)
                    .center_y()
                )
                .into(),
            );
            col_vec.push(
                row!(
                    container(appearance::icon('󰈼')),
                    container(
                        text_input("Input deadline here", &self.ddl_cache)
                            .on_input(|input| {
                                Message::TodoMessage(
                                    self.id.to_owned(),
                                    TodoMessage::Edit(EditMessage::Deadline(input)),
                                )
                            })
                            .width(350)
                    )
                    .height(height)
                    .center_y()
                )
                .into(),
            );
            col_vec.push(
                row!(
                    container(appearance::icon('󰓻')).height(height).center_y(),
                    container(
                        text_input(
                            "Separate tags by space",
                            &(if todo.tags.is_empty() {
                                String::new()
                            } else {
                                format!(
                                    "{} ",
                                    util::join_str_with(
                                        todo.tags.iter().map(|e| e.as_str()).collect(),
                                        " "
                                    )
                                )
                            })
                        )
                        .on_input(|input| {
                            Message::TodoMessage(
                                self.id.to_owned(),
                                TodoMessage::Edit(EditMessage::Tags(input)),
                            )
                        })
                        .width(350)
                    )
                    .height(height)
                    .center_y()
                )
                .into(),
            );

            self_vec.push(column(col_vec).into());

            right_vec.push(horizontal_space(8.5).into());

            // Right side controls for editing
            let mut controls_vec: Vec<Element<'_, Message, Renderer>> = Vec::new();
            controls_vec.push(
                container(
                    button(appearance::icon('󰸞').style(theme::Text::Color(app.style_sheet().gray)))
                        .style(theme::Button::Text)
                        .on_press(Message::TodoMessage(
                            self.id.to_owned(),
                            TodoMessage::Edit(EditMessage::ToggleEdit),
                        )),
                )
                .height(height)
                .center_y()
                .into(),
            );

            controls_vec.push(
                container(
                    button(appearance::icon('󰙅').style(theme::Text::Color(app.style_sheet().gray)))
                        .style(theme::Button::Text)
                        .on_press(Message::TodoMessage(
                            self.id.to_owned(),
                            TodoMessage::Edit(EditMessage::ToggleSelectChildren),
                        )),
                )
                .style(if app.dep_selection.is_none() {
                    theme::Container::Transparent
                } else {
                    theme::Container::Box
                })
                .height(height)
                .center_y()
                .into(),
            );
            controls_vec.push(
                container(
                    button(appearance::icon('󰩹').style(theme::Text::Color(app.style_sheet().gray)))
                        .style(theme::Button::Text)
                        .on_press(Message::TodoMessage(
                            self.id.to_owned(),
                            TodoMessage::Delete,
                        )),
                )
                .height(height)
                .center_y()
                .into(),
            );

            right_vec.push(column(controls_vec).into());
        }

        right_vec.push(horizontal_space(22.5).into());

        self_vec.push(
            container(row(left_vec))
                .align_x(alignment::Horizontal::Left)
                .width(Length::FillPortion(3))
                .into(),
        );

        self_vec.push(
            container(row(right_vec))
                .align_x(alignment::Horizontal::Right)
                .width(Length::FillPortion(1))
                .into(),
        );

        let mut vec: Vec<(u16, Vec<Element<'_, Message, Renderer>>)> = Vec::new();

        vec.push((
            12,
            if self.editing {
                self_vec
            } else {
                vec![column({
                    let mut column_items = vec![row(self_vec).into()];

                    if !todo.metadata.details.is_empty() {
                        column_items.push(
                            row![
                                horizontal_space(60),
                                text(todo.metadata.details.clone())
                                    .style(theme::Text::Color(app.style_sheet().gray))
                                    .size(13.5)
                            ]
                            .into(),
                        );
                    }

                    column_items
                })
                .into()]
            },
        ));

        if self.expanded {
            for todo_id in app.instance.children_once(self.id) {
                if app.range.contains(&todo_id) && {
                    let mut b = true;
                    for c in app.instance.children(self.id) {
                        if app.instance.all_deps(todo_id).contains(&c) {
                            b = false;
                        }
                    }
                    b
                } {
                    for v in app.state(todo_id).unwrap().view(app) {
                        vec.push((v.0 + 25, v.1));
                    }
                }
            }
        }
        vec
    }
}

fn completion_state_view(id: u64, instance: &TodoInstance) -> char {
    let todo = instance.get(id).unwrap();
    if todo.completed {
        if instance.children_once(id).is_empty() {
            '󰄲'
        } else {
            '󰗠'
        }
    } else if instance.children_once(id).is_empty() {
        '󰄱'
    } else {
        util::progression_char((instance.weight(id, true) * 100) / instance.weight(id, false))
    }
}
