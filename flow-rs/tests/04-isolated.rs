mod nodes_ext;

use anyhow::Result;
use flow_rs::prelude::*;

#[rt::test]
async fn test_unused() -> Result<()> {
    let mut graph = Builder::default()
        .template(
            r#"
main="test"
[[graphs]]
name="test"
[[graphs.nodes]]
name="t"
ty="Transform"
        "#
            .to_owned(),
        )
        .build()?;
    graph.start().await?;
    Ok(())
}

#[rt::test]
async fn test_isolated() -> Result<()> {
    let mut graph = Builder::default()
        .template(
            r#"
main="test"
[[graphs]]
name="test"
nodes=[
    {name="a", ty="Isolated"},
]
        "#
            .to_owned(),
        )
        .build()?;
    graph.start().await?;
    Ok(())
}

#[rt::test]
async fn test_isolated_in_global() -> Result<()> {
    let mut graph = Builder::default()
        .template(
            r#"
main="test"
[[nodes]]
name="a"
ty="IsolatedNever"
[[graphs]]
name="test"
[[graphs.inputs]]
name="inp"
cap=1
ports=["t:inp"]
[[graphs.outputs]]
name="out"
cap=1
ports=["t:out"]
[[graphs.nodes]]
name="t"
ty="Transform"
        "#
            .to_owned(),
        )
        .build()?;
    graph.start().await?;
    Ok(())
}
