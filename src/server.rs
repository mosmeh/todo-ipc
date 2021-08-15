use todo_ipc::{Connection, IpcError, Request, Response, Task, IPC_ENDPOINT};

use anyhow::Result;
use futures::StreamExt;
use parity_tokio_ipc::Endpoint;
use std::cell::RefCell;
use tokio::io::{AsyncRead, AsyncWrite};

struct State {
    list: Vec<Task>,
}

async fn handle_conn(state: &RefCell<State>, conn: impl AsyncRead + AsyncWrite) -> Result<()> {
    let mut conn = Connection::new(conn);

    while let Ok(req) = conn.recv().await {
        println!("<- {:?}", req);

        let list = &mut state.borrow_mut().list;
        let res = match req {
            Request::Add(description) => {
                list.push(Task {
                    description,
                    checked: false,
                });
                Ok(None)
            }
            Request::Remove(i) => {
                if i < list.len() {
                    list.remove(i);
                    Ok(None)
                } else {
                    Err(IpcError::TaskNotFound(i))
                }
            }
            Request::Check(i) => {
                if let Some(task) = list.get_mut(i) {
                    task.checked = true;
                    Ok(None)
                } else {
                    Err(IpcError::TaskNotFound(i))
                }
            }
            Request::Uncheck(i) => {
                if let Some(task) = list.get_mut(i) {
                    task.checked = false;
                    Ok(None)
                } else {
                    Err(IpcError::TaskNotFound(i))
                }
            }
            Request::List => Ok(Some(Response::List(list.clone()))),
        };
        println!("-> {:?}", res);
        conn.send(&res).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = RefCell::new(State { list: Vec::new() });
    Endpoint::new(IPC_ENDPOINT.into())
        .incoming()?
        .for_each_concurrent(None, |conn| async {
            if let Ok(conn) = conn {
                let _ = handle_conn(&state, conn).await;
            }
        })
        .await;
    Ok(())
}
