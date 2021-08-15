use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{
    AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufStream, ReadHalf, WriteHalf,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Bincode(#[from] bincode::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub const IPC_ENDPOINT: &str = {
    #[cfg(windows)]
    {
        concat!(r"\\.\pipe\", env!("CARGO_PKG_NAME"), "-ipc")
    }
    #[cfg(not(windows))]
    {
        concat!("/tmp/", env!("CARGO_PKG_NAME"), "-ipc")
    }
};

pub struct Connection<C> {
    reader: ReadHalf<BufStream<C>>,
    writer: WriteHalf<BufStream<C>>,
}

impl<C> Connection<C>
where
    C: AsyncRead + AsyncWrite,
{
    pub fn new(conn: C) -> Self {
        let conn = BufStream::new(conn);
        let (reader, writer) = tokio::io::split(conn);
        Self { reader, writer }
    }

    pub async fn send<T: Serialize>(&mut self, data: &T) -> Result<()> {
        let buf = bincode::DefaultOptions::new().serialize(data)?;
        self.writer.write_u64(buf.len() as u64).await?;
        self.writer.write_all(&buf).await?;
        self.writer.flush().await?;
        Ok(())
    }

    pub async fn recv<T: DeserializeOwned>(&mut self) -> Result<T> {
        let len = self.reader.read_u64().await?;
        let mut buf = vec![0u8; len as usize];
        self.reader.read_exact(&mut buf).await?;
        let data = bincode::DefaultOptions::new().deserialize(&buf)?;
        Ok(data)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Add(String),
    Remove(usize),
    Check(usize),
    Uncheck(usize),
    List,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    List(Vec<Task>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub description: String,
    pub checked: bool,
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum IpcError {
    #[error("No task with ID {0} found")]
    TaskNotFound(usize),
}
