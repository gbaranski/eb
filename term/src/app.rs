use crossterm::event::DisableMouseCapture;
use crossterm::event::Event;
use crossterm::event::EventStream;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal;
use eb_rpc::client;
use eb_rpc::Client;
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

pub struct App<B: Backend, C: Client> {
    terminal: Terminal<B>,
    client: C,
    should_exit: bool,
    content: String,
    cursor: usize,
}

impl<C: Client> App<CrosstermBackend<Stdout>, C> {
    pub fn new(client: C) -> Result<Self, anyhow::Error> {
        terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = tui::Terminal::new(backend)?;
        Ok(Self {
            terminal,
            should_exit: false,
            client,
            content: String::new(),
            cursor: 0,
        })
    }
}

impl<B: Backend, C: Client + Send + Sync> App<B, C> {
    fn render(&mut self) -> Result<(), anyhow::Error> {
        let content = self.content.to_string();
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)])
                .split(f.size());
            let span = tui::text::Span::raw(content);
            let paragraph = Paragraph::new(span);
            f.render_widget(paragraph, chunks[0])
        })?;
        Ok(())
    }

    fn exit(&mut self) -> Result<(), anyhow::Error> {
        self.should_exit = true;
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), anyhow::Error> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Esc => {
                    self.exit()?;
                }
                // KeyCode::Backspace => {
                //     if self.char_idx > 0 {
                //         self.content.remove(self.char_idx - 1..self.char_idx);
                //         self.char_idx -= 1;
                //     }
                // }
                KeyCode::Char(char) => {
                    self.client
                        .insert(client::insert::Request {
                            content: char.to_string(),
                            cursor: self.cursor,
                        })
                        .await??;
                }
                _ => unimplemented!(),
            },
            _ => {}
        };
        Ok(())
    }

    pub async fn run(mut self) -> Result<(), anyhow::Error> {
        let mut events = EventStream::new();
        self.terminal.clear()?;
        tokio::spawn(async move {});
        loop {
            if self.should_exit {
                break;
            }
            self.render()?;
            tokio::select! {
                biased;
                event = events.next() => {
                    if let Some(Ok(event)) = event {
                        self.handle_event(event).await?;
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
