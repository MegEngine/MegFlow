mod nodes_ext;

use anyhow::Result;
use flow_rs::prelude::*;
use std::io::Write;

#[rt::test]
async fn test_isolated() -> Result<()> {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    write!(
        file,
        "{}",
        r#"
main="test"
[[graphs]]
name="test"
nodes=[
    {name="a", ty="Isolated"},
]
        "#
    )?;
    let mut graph = load(None, file.path())?;
    graph.start().await;
    Ok(())
}

#[rt::test]
async fn test_isolated_in_global() -> Result<()> {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    write!(
        file,
        "{}",
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
    )?;
    let mut graph = load(None, file.path())?;
    graph.start().await;
    Ok(())
}
