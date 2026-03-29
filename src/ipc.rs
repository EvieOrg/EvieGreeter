use greetd_ipc::{AuthMessageType, ErrorType, Request, Response};
use greetd_ipc::codec::TokioCodec;
use tokio::net::UnixStream;

pub struct GreetdClient {
    stream: UnixStream,
}

#[derive(Debug)]
pub enum IpcError {
    Io(std::io::Error),
    Codec(greetd_ipc::codec::Error),
    AuthError(String),
    Unexpected(String),
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcError::Io(e) => write!(f, "IO error: {}", e),
            IpcError::Codec(e) => write!(f, "Codec error: {}", e),
            IpcError::AuthError(s) => write!(f, "Authentication failed: {}", s),
            IpcError::Unexpected(s) => write!(f, "Unexpected: {}", s),
        }
    }
}

impl From<std::io::Error> for IpcError {
    fn from(e: std::io::Error) -> Self {
        IpcError::Io(e)
    }
}

impl From<greetd_ipc::codec::Error> for IpcError {
    fn from(e: greetd_ipc::codec::Error) -> Self {
        IpcError::Codec(e)
    }
}

impl GreetdClient {
    pub async fn connect() -> Result<Self, IpcError> {
        let sock = std::env::var("GREETD_SOCK")
            .unwrap_or_else(|_| "/run/greetd.sock".into());
        let stream = UnixStream::connect(&sock).await?;
        Ok(Self { stream })
    }

    pub async fn create_session(&mut self, username: &str) -> Result<Option<String>, IpcError> {
        self.send(Request::CreateSession {
            username: username.to_string(),
        })
        .await?;

        match self.recv().await? {
            Response::AuthMessage {
                auth_message_type: AuthMessageType::Secret,
                auth_message,
            }
            | Response::AuthMessage {
                auth_message_type: AuthMessageType::Visible,
                auth_message,
            } => Ok(Some(auth_message)),
            Response::Success => Ok(None),
            Response::Error {
                error_type: ErrorType::AuthError,
                description,
            } => Err(IpcError::AuthError(description)),
            Response::Error { description, .. } => Err(IpcError::Unexpected(description)),
            _ => Err(IpcError::Unexpected("unexpected response".into())),
        }
    }

    pub async fn post_auth_message_response(
        &mut self,
        response: Option<String>,
    ) -> Result<Option<String>, IpcError> {
        self.send(Request::PostAuthMessageResponse { response })
            .await?;

        match self.recv().await? {
            Response::AuthMessage {
                auth_message_type: AuthMessageType::Secret,
                auth_message,
            }
            | Response::AuthMessage {
                auth_message_type: AuthMessageType::Visible,
                auth_message,
            } => Ok(Some(auth_message)),
            Response::Success => Ok(None),
            Response::Error {
                error_type: ErrorType::AuthError,
                description,
            } => Err(IpcError::AuthError(description)),
            Response::Error { description, .. } => Err(IpcError::Unexpected(description)),
            _ => Err(IpcError::Unexpected("unexpected response".into())),
        }
    }

    pub async fn start_session(&mut self, cmd: Vec<String>) -> Result<(), IpcError> {
        self.send(Request::StartSession { cmd, env: vec![] })
            .await?;

        match self.recv().await? {
            Response::Success => Ok(()),
            Response::Error { description, .. } => Err(IpcError::Unexpected(description)),
            _ => Err(IpcError::Unexpected("unexpected response to StartSession".into())),
        }
    }

    #[allow(dead_code)]
    pub async fn cancel_session(&mut self) -> Result<(), IpcError> {
        let _ = self.send(Request::CancelSession).await;
        Ok(())
    }

    async fn send(&mut self, req: Request) -> Result<(), IpcError> {
        req.write_to(&mut self.stream).await.map_err(IpcError::from)
    }

    async fn recv(&mut self) -> Result<Response, IpcError> {
        Response::read_from(&mut self.stream)
            .await
            .map_err(IpcError::from)
    }
}
