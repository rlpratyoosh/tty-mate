use crossterm::{
    event::{
        self,
        KeyEventKind,
        KeyEvent,
        Event,
    }
};
use ratatui::{
    DefaultTerminal,
    Frame,
};
use std::time::Duration;
use crate::ui::{
    game::Game,
    menu::Menu,
    multiplayer::MultiplayerGame,
};
use tty_mate_api::{ServerMessage, GameError, ClientMessage};
use tokio::{
    task,
    sync::mpsc,
    net::TcpStream,
    io::{BufReader, AsyncBufReadExt, AsyncWriteExt},
};

pub struct App {
    game: Game,
    multiplayer_game: Option<MultiplayerGame>,
    menu: Menu,
    app_state: AppState,
    exit: bool,
    tx: mpsc::UnboundedSender<ClientEvent>,
    rx: mpsc::UnboundedReceiver<ClientEvent>,
}

pub enum AppState {
    Game,
    Menu,
    Multiplayer,
}

pub enum AppAction {
    None,
    StartGame,
    StartMultiplayer,
    QuitToMenu,
    Exit,
}

enum ClientEvent {
    Input(KeyEvent),
    Network(ServerMessage),
    NetworkError(GameError),
}

impl App {
    pub fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        App {
            game: Game::default(),
            multiplayer_game: None,
            menu: Menu::default(),
            app_state: AppState::Menu,
            exit: false,
            tx,
            rx,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {

        let tx_input = self.tx.clone();
        task::spawn_blocking(move || {
            loop {
                if event::poll(Duration::from_millis(250)).unwrap_or(false) {
                    if let Ok(Event::Key(key_event)) = event::read() {
                        if key_event.kind == KeyEventKind::Press {
                            if tx_input.send(ClientEvent::Input(key_event)).is_err() {
                                break; 
                            }
                        }
                    }
                } else {
                    if tx_input.is_closed() {
                        break;
                    }
                }
            }
        });

        while !self.exit {
            terminal.draw(|f| self.draw(f))?;
            match self.rx.recv().await {
                Some(ClientEvent::Input(key_event)) => self.handle_key_event(key_event),
                Some(ClientEvent::Network(message)) => {
                    if let Some(game) = self.multiplayer_game.as_mut() {
                        game.handle_network_event(message);
                    }
                }
                Some(ClientEvent::NetworkError(error)) => {
                     if let Some(game) = self.multiplayer_game.as_mut() {
                        game.handle_network_error(error);
                    }
                }
                None => {},
            }
        }
        Ok(())
    }

    fn draw(&self, f: &mut Frame) {
        match self.app_state {
            AppState::Menu => f.render_widget(&self.menu, f.area()),
            AppState::Game => f.render_widget(&self.game, f.area()),
            AppState::Multiplayer => {
                if let Some(game) = self.multiplayer_game.as_ref() {
                    f.render_widget(game, f.area());
                }
            }
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let action = match self.app_state {
            AppState::Menu => self.menu.handle_key_event(key_event),
            AppState::Game => self.game.handle_key_event(key_event),
            AppState::Multiplayer => {
                if let Some(game) = self.multiplayer_game.as_mut() {
                    game.handle_key_event(key_event)
                } else {
                    AppAction::None
                }
            }
,
        };

        match action {
            AppAction::StartGame => { 
                self.game = Game::default();
                self.app_state = AppState::Game;
            },
            AppAction::StartMultiplayer => {
                let (tx, mut rx) = mpsc::unbounded_channel::<ClientMessage>();

                self.multiplayer_game = Some(MultiplayerGame::new(tx));
                self.app_state = AppState::Multiplayer;
                let tx_net = self.tx.clone();

                tokio::spawn(async move {
                    let mut socket = match TcpStream::connect("127.0.0.1:8080").await {
                        Ok(s) => s,
                        Err(_) => {
                            let _ = tx_net.send(ClientEvent::NetworkError(GameError::NoGameFound));
                            return;
                        }
                    };

                    let (tcp_reader, mut tcp_writer) = socket.split();
                    let mut tcp_reader = BufReader::new(tcp_reader).lines();

                    loop {
                        tokio::select! {
                            result = tcp_reader.next_line() => {
                                let Ok(Some(line)) = result else {
                                    break;
                                };
                                let Ok(server_message) = ServerMessage::parse(&line) else {
                                    match GameError::parse(&line) {
                                        Ok(game_error) => {
                                            let _ = tx_net.send(ClientEvent::NetworkError(game_error));
                                        },
                                        Err(e) => {
                                             let _ = tx_net.send(ClientEvent::NetworkError(e));
                                        },
                                    }
                                    continue;
                                };
                                let _ = tx_net.send(ClientEvent::Network(server_message));
                            }

                            Some(client_msg) = rx.recv() => {
                                let msg_str = client_msg.to_string();
                                if let Err(_) = tcp_writer.write_all(msg_str.as_bytes()).await {
                                    break;
                                }
                            }
                        }
                    }
                });
            },
            AppAction::QuitToMenu => {
                self.app_state = AppState::Menu;
                self.multiplayer_game = None;
            }
            AppAction::Exit => self.exit(),
            AppAction::None => {},
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

