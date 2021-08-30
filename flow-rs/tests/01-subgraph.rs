mod nodes_ext;

use anyhow::Result;
use flow_rs::prelude::*;

#[rt::test]
async fn test_error() -> Result<()> {
    let mut graph = load(
        None,
        r#"
main="test"
[[graphs]]
name="sub"
nodes=[
    {name="a", ty="Transform"},
    {name="b", ty="ErrorOpr"},
    {name="c", ty="Transform"}
]
inputs=[{name="inp",cap=1,ports=["a:inp"]}]
outputs=[{name="out",cap=1,ports=["c:out"]}]
connections=[
    {cap=1,ports=["a:out", "b:inp"]},
    {cap=1,ports=["b:out", "c:inp"]}
]
[[graphs]]
name="test"
nodes=[
    {name="a", ty="NeverOpr"},
    {name="b", ty="sub"},
    {name="c", ty="NoopConsumer"},
]
connections=[
    {cap=1,ports=["a:out", "b:inp"]},
    {cap=1,ports=["b:out", "c:inp"]}
]
        "#,
    )?;
    graph.start(None).await;
    Ok(())
}

#[rt::test]
async fn test_basis() -> Result<()> {
    let _ = load(
        None,
        r#"
main="test"
[[graphs]]
name="sub"
nodes=[{name="b",ty="BinaryOpr"}]
inputs=[
    {name="a",cap=1,ports=["b:a"]},
    {name="b",cap=1,ports=["b:b"]}
]
outputs=[{name="c",cap=1,ports=["b:c"]}]
[[graphs]]
name="test"
inputs=[{name="inp",cap=1,ports=["t1:inp","t2:inp"]}]
outputs=[{name="out",cap=1,ports=["t3:out"]}]
connections=[
    {cap=1,ports=["t1:out", "sub:a"]},
    {cap=1,ports=["t2:out", "sub:b"]},
    {cap=1,ports=["t3:inp", "sub:c"]}
]
nodes=[
    {name="sub",ty="sub"},
    {name="t1",ty="Transform"},
    {name="t2",ty="Transform"},
    {name="t3",ty="Transform"}
]
        "#,
    )?;

    Ok(())
}

#[rt::test]
async fn test_empty() -> Result<()> {
    let _ = load(
        None,
        r#"
main="test"
nodes=[{name="gb",ty="BinaryOpr"}]
[[graphs]]
name="sub"
inputs=[
    {name="a",cap=1,ports=["gb:a"]},
    {name="b",cap=1,ports=["gb:b"]}
]
outputs=[{name="c",cap=1,ports=["gb:c"]}]
[[graphs]]
name="test"
inputs=[{name="inp",cap=1,ports=["t1:inp","t2:inp"]}]
outputs=[{name="out",cap=1,ports=["t3:out"]}]
connections=[
    {cap=1,ports=["t1:out", "sub:a"]},
    {cap=1,ports=["t2:out", "sub:b"]},
    {cap=1,ports=["t3:inp", "sub:c"]}
]
nodes=[
    {name="sub",ty="sub"},
    {name="t1",ty="Transform"},
    {name="t2",ty="Transform"},
    {name="t3",ty="Transform"}
]
        "#,
    )?;

    Ok(())
}
