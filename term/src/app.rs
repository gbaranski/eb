use crossterm::event::DisableMouseCapture;
use crossterm::event::Event;
use crossterm::event::EventStream;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal;
use eb_client::Client;
use eb_core::client;
use eb_core::server;
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
    client: Client,
    should_exit: bool,
    content: String,
}

impl App<CrosstermBackend<Stdout>> {
    pub fn new(client: Client) -> Result<Self, anyhow::Error> {
        terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = tui::Terminal::new(backend)?;
        Ok(Self {
            terminal,
            should_exit: false,
            content: String::new(),
            client,
        })
    }
}

impl<B: Backend> App<B> {
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
                        .send(&client::Message::Insert {
                            content: char.to_string(),
                        })
                        .await?;
                }
                _ => unimplemented!(),
            },
            _ => {}
        };
        Ok(())
    }

    async fn handle_message(&mut self, message: server::Message) -> Result<(), anyhow::Error> {
        tracing::debug!("Received message: {:?}", message);
        match message {
            server::Message::Insert { content } => {
                self.content += &content;
            }
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
                message = self.client.recv() => {
                    if let Some(message) = message? {
                        self.handle_message(message).await?
                    } else {
                        return Ok(())
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
