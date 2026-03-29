use crate::ipc::{GreetdClient, IpcError};
use crate::session::Session;
use crate::ui::login_card::LoginMsg;
use async_channel::Receiver;

#[derive(Debug, Clone)]
pub enum GreeterEvent {
    ShowError(String),
    ClearError,
    ClearPassword,
    SessionStarted,
}

/// State machine del greeter. Gira in un task tokio separato,
/// comunica con la UI tramite canali async.
pub struct Greeter {
    rx: Receiver<LoginMsg>,
    event_tx: async_channel::Sender<GreeterEvent>,
    sessions: Vec<Session>,
    current_user: String,
    current_session_idx: usize,
}

impl Greeter {
    pub fn new(
        rx: Receiver<LoginMsg>,
        event_tx: async_channel::Sender<GreeterEvent>,
        sessions: Vec<Session>,
    ) -> Self {
        Self {
            rx,
            event_tx,
            sessions,
            current_user: String::new(),
            current_session_idx: 0,
        }
    }

    pub async fn run(mut self) {
        while let Ok(msg) = self.rx.recv().await {
            match msg {
                LoginMsg::UserSelected(user) => {
                    self.current_user = user;
                }
                LoginMsg::SessionSelected(idx) => {
                    self.current_session_idx = idx;
                }
                LoginMsg::PasswordSubmit(password) => {
                    let _ = self.event_tx.send(GreeterEvent::ClearError).await;
                    self.authenticate(password).await;
                }
            }
        }
    }

    async fn authenticate(&self, password: String) {
        let result = self.try_login(&password).await;
        match result {
            Ok(()) => {
                let _ = self.event_tx.send(GreeterEvent::SessionStarted).await;
            }
            Err(e) => {
                let _ = self
                    .event_tx
                    .send(GreeterEvent::ShowError(e.to_string()))
                    .await;
                let _ = self.event_tx.send(GreeterEvent::ClearPassword).await;
            }
        }
    }

    async fn try_login(&self, password: &str) -> Result<(), IpcError> {
        let mut client = GreetdClient::connect().await?;

        // CreateSession
        let prompt = client.create_session(&self.current_user).await?;

        // Se c'è un auth message (password prompt), rispondi
        if prompt.is_some() {
            client
                .post_auth_message_response(Some(password.to_string()))
                .await?;
        }

        // StartSession
        let session = self
            .sessions
            .get(self.current_session_idx)
            .ok_or_else(|| IpcError::Unexpected("no session selected".into()))?;

        // Per la sessione TTY switcha al VT 5 prima di avviare
        if session.name == "TTY" {
            let _ = std::process::Command::new("chvt").arg("5").status();
        }

        client.start_session(session.cmd.clone()).await?;
        Ok(())
    }
}
