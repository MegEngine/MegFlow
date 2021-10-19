use anyhow::Result;
use flow_rs::prelude::*;
use futures_util::{select, stream::FuturesUnordered, StreamExt};
use toml::value::Table;

#[inputs(a, b)]
#[outputs(c)]
#[derive(Node, Actor, Default)]
struct BinaryOpr {}

impl BinaryOpr {
    fn new(_name: String, _args: &Table) -> BinaryOpr {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}
    async fn exec(&mut self, _: &Context) -> Result<()> {
        let mut recv_a = FuturesUnordered::new();
        recv_a.push(self.a.recv_any());
        let mut recv_b = FuturesUnordered::new();
        recv_b.push(self.b.recv_any());
        loop {
            select! {
                a = recv_a.select_next_some() => {
                    if let Ok(a) = a {
                        self.c.send_any(a).await.ok();
                        recv_a.push(self.a.recv_any());
                    }
                }
                b = recv_b.select_next_some() => {
                    if let Ok(b) = b {
                        self.c.send_any(b).await.ok();
                        recv_b.push(self.b.recv_any());
                    }
                }
                complete => break,
            }
        }
        Ok(())
    }
}

node_register!("BinaryOpr", BinaryOpr);

#[inputs(inp)]
#[outputs(out)]
#[derive(Node, Actor, Default)]
struct ErrorOpr {}

impl ErrorOpr {
    fn new(_name: String, _: &Table) -> ErrorOpr {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}
    async fn exec(&mut self, _: &Context) -> Result<()> {
        Err(anyhow::anyhow!("error"))
    }
}

node_register!("ErrorOpr", ErrorOpr);

#[inputs]
#[outputs(out)]
#[derive(Node, Actor, Default)]
struct NeverOpr {}

impl NeverOpr {
    fn new(_name: String, _: &Table) -> NeverOpr {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}
    async fn exec(&mut self, ctx: &Context) -> Result<()> {
        ctx.wait().await;
        Ok(())
    }
}

node_register!("NeverOpr", NeverOpr);

#[inputs]
#[outputs]
#[derive(Node, Actor, Default)]
struct Isolated {}

impl Isolated {
    fn new(_name: String, _: &Table) -> Self {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}
    async fn exec(&mut self, _: &Context) -> Result<()> {
        Ok(())
    }
}

node_register!("Isolated", Isolated);

#[inputs]
#[outputs]
#[derive(Node, Actor, Default)]
struct IsolatedNever {}

impl IsolatedNever {
    fn new(_name: String, _: &Table) -> Self {
        Default::default()
    }

    async fn initialize(&mut self, _: ResourceCollection) {}
    async fn finalize(&mut self) {}
    async fn exec(&mut self, ctx: &Context) -> Result<()> {
        ctx.wait().await;
        Ok(())
    }
}

node_register!("IsolatedNever", IsolatedNever);
