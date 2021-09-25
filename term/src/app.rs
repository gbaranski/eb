use crossterm::event::DisableMouseCapture;
use crossterm::event::Event;
use crossterm::event::EventStream;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal;
use futures::StreamExt;
use std::io::Stdout;
use std::io::Write;
use tui::backend::Backend;
use tui::backend::CrosstermBackend;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::widgets::Paragraph;
use tui::Terminal;

pub struct App<B: Backend> {
    terminal: Terminal<B>,
    should_exit: bool,
}

impl App<CrosstermBackend<Stdout>> {
    pub fn new() -> Result<Self, anyhow::Error> {
        terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = tui::Terminal::new(backend)?;
        Ok(Self {
            terminal,
            should_exit: false,
        })
    }
}

impl<B: Backend> App<B> {
    fn render(&mut self) -> Result<(), anyhow::Error> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(10)
                .constraints([Constraint::Percentage(100)])
                .split(f.size());
            let span = tui::text::Span::raw("test");
            let paragraph = Paragraph::new(span);
            f.render_widget(paragraph, chunks[0])
        })?;
        Ok(())
    }

    fn exit(&mut self) -> Result<(), anyhow::Error> {
        self.should_exit = true;
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<(), anyhow::Error> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Esc => {
                    self.exit()?;
                }
                KeyCode::Char(char) => {}
                _ => {}
            },
            _ => {}
        };
        Ok(())
    }

    pub async fn run(mut self) -> Result<(), anyhow::Error> {
        let mut events = EventStream::new();
        self.terminal.clear()?;
        loop {
            if self.should_exit {
                break;
            }
            self.render()?;
            tokio::select! {
                event = events.next() => {
                    if let Some(Ok(event)) = event {
                        self.handle_event(event)?;
                    }
                }
            };
        }

        let mut stdout = std::io::stdout();
        // reset cursor shape
        write!(stdout, "\x1B[2 q")?;
        execute!(stdout, DisableMouseCapture)?;
        execute!(stdout, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}
