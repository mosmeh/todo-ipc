use todo_ipc::{Connection, IpcError, Request, Response, IPC_ENDPOINT};

use anyhow::Result;
use parity_tokio_ipc::Endpoint;
use std::num::NonZeroUsize;
use structopt::StructOpt;

#[derive(StructOpt)]
enum Command {
    Add {
        description: Vec<String>,
    },
    #[structopt(alias = "rm", alias = "delete", alias = "del")]
    Remove {
        id: NonZeroUsize,
    },
    #[structopt(alias = "do")]
    Check {
        id: NonZeroUsize,
    },
    #[structopt(alias = "undo")]
    Uncheck {
        id: NonZeroUsize,
    },
    #[structopt(alias = "ls")]
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Command::from_args();

    let conn = Endpoint::connect(IPC_ENDPOINT).await?;
    let mut conn = Connection::new(conn);

    let req = match cmd {
        Command::Add { description } => Request::Add(description.join(" ")),
        Command::Remove { id } => Request::Remove(id.get() - 1),
        Command::Check { id } => Request::Check(id.get() - 1),
        Command::Uncheck { id } => Request::Uncheck(id.get() - 1),
        Command::List => Request::List,
    };
    conn.send(&req).await?;

    let res: Result<Option<Response>, IpcError> = conn.recv().await?;
    if let Some(res) = res? {
        match res {
            Response::List(list) => {
                for (i, task) in list.into_iter().enumerate() {
                    println!(
                        "{:3} [{}] {}",
                        i + 1,
                        if task.checked { "x" } else { " " },
                        task.description
                    );
                }
            }
        }
    }

    Ok(())
}
