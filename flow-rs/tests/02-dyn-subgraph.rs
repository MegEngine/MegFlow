mod nodes_ext;

use anyhow::Result;
use flow_rs::prelude::*;

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
    {name="t1",ty="DynOutTransform"},
    {name="t2",ty="DynOutTransform"},
    {name="t3",ty="DynInTransform"}
]
        "#,
    )?;

    Ok(())
}
