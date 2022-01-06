mod nodes_ext;

use anyhow::Result;
use flow_rs::prelude::*;

#[rt::test]
async fn test_basis() -> Result<()> {
    let mut graph = Builder::default()
        .template(
            r#"
main="test"
nodes = [{name="sub", ty="sub"}]
[[graphs]]
name="sub"
nodes=[{name="b",ty="BinaryOpr"}]
inputs=[
    {name="a",cap=1,ports=["b:a"]},
    {name="b",cap=1,ports=["b:b"]}
]
outputs=[{name="c",cap=2,ports=["b:c"]}]
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
    {name="t1",ty="Transform"},
    {name="t2",ty="Transform"},
    {name="t3",ty="Transform"}
]
        "#
            .to_owned(),
        )
        .build()?;

    let inp = graph.input("inp").unwrap();
    let out = graph.output("out").unwrap();
    let handle = graph.start();

    inp.send(Envelope::new(1usize)).await.ok();
    inp.close();
    assert!(out.recv::<usize>().await.is_ok());
    assert!(out.recv::<usize>().await.is_ok());
    assert!(out.recv::<usize>().await.is_err());
    handle.await?;

    Ok(())
}

#[test]
#[should_panic]
fn test_empty() {
    let _ = Builder::default()
        .template(
            r#"
main="test"
nodes=[
    {name="gb",ty="BinaryOpr"},
    {name="sub",ty="sub"}
]
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
    {name="t1",ty="Transform"},
    {name="t2",ty="Transform"},
    {name="t3",ty="Transform"}
]
        "#
            .to_owned(),
        )
        .build()
        .unwrap();
}
