/**
 * \file flow-rs/src/config/parser.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;
use templar::*;
use unstructured::*;

fn get_num<'a, T>(data: &'a InnerData) -> T
where
    T: From<&'a Number>,
{
    match data {
        Unstructured::Number(n) => n.into(),
        _ => unreachable!(),
    }
}

fn range_impl(start: i64, end: i64, step: i64) -> Vec<i64> {
    (start..end).step_by(step as usize).collect()
}

fn range(n: Data) -> Data {
    let list = match &*n {
        Unstructured::Number(end) => range_impl(0, end.into(), 1),
        Unstructured::Seq(list) => match &list[..] {
            [start, end] => {
                let start = get_num(start);
                let end = get_num(end);
                range_impl(start, end, 1)
            }
            [start, end, step] => {
                let start = get_num(start);
                let end = get_num(end);
                let step = get_num(step);
                assert!(step > 0);
                range_impl(start, end, step)
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    Data::new(Unstructured::Seq(
        list.into_iter()
            .map(|x| Unstructured::Number(Number::I64(x)))
            .collect(),
    ))
}

lazy_static::lazy_static! {
    pub static ref TEMPLAR: Templar = {
        let mut builder = TemplarBuilder::default();
        builder.add_function("range", range);
        builder.build()
    };
}

pub trait Parser<'a>: Deserialize<'a> {
    fn from_file(template: &Path, dynamic: Option<&Path>) -> Result<Self>;
    fn from_str(template: &str, dynamic: Option<&str>) -> Result<Self>;
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use flow_rs::prelude::Parser;
    use serde::Deserialize;
    use std::collections::*;
    use std::io::Write;
    use std::path::PathBuf;

    #[derive(Deserialize, Debug, Parser)]
    struct Config {
        include: Vec<PathBuf>,
        a: i32,
        b: String,
        c: Vec<i32>,
        d: HashSet<i32>,
        e: BTreeMap<String, i32>,
    }

    #[test]
    fn test_basis() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let file1_path = temp_dir.path().join("a.toml");
        let file2_path = temp_dir.path().join("b.toml");
        let mut file1 = std::fs::File::create(&file1_path)?;
        write!(
            file1,
            "{}",
            r#"
include=["./b.toml"]
a=1
b="string in a"
c=[1,2,3]
d=[4,5,6]
e={a=1, b=2}"#
        )?;
        let mut file2 = std::fs::File::create(&file2_path)?;
        write!(
            file2,
            "{}",
            r#"
include=[]
a=2
b="string in b"
c=[4,5,6]
d=[1,2,3]
e={c=3, b=4}"#
        )?;
        let config: Config = Parser::from_file(&file1_path, None)?;
        assert_eq!(config.a, 1);
        assert_eq!(config.b, "string in a");
        assert_eq!(config.c, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(
            config
                .d
                .difference(&(1..=6).collect::<HashSet<i32>>())
                .count(),
            0
        );
        assert_eq!(config.e.len(), 3);
        assert_eq!(config.e["a"], 1);
        assert_eq!(config.e["b"], 2);
        assert_eq!(config.e["c"], 3);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_cyclic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file1_path = temp_dir.path().join("a.toml");
        let mut file1 = std::fs::File::create(&file1_path).unwrap();
        write!(
            file1,
            "{}",
            r#"
include=["./a.toml"]
a=1
b="string in a"
c=[1,2,3]
d=[4,5,6]
e={a=1, b=2}"#
        )
        .unwrap();
        let _: Config = Parser::from_file(&file1_path, None).unwrap();
    }

    #[test]
    fn test_template() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let file1_path = temp_dir.path().join("a.toml");
        let file2_path = temp_dir.path().join("dyn.toml");
        let mut file1 = std::fs::File::create(&file1_path)?;
        write!(
            file1,
            "{}",
            r#"
include=[]
{{ a = 1 -}}
{{ b = "string" -}}
a={{a}}
b="{{b}}"
c = [{% for i in List %} {{- i+1 -}}, {% end for %}]
d = [{% for i in List %} {{- i*2 -}}, {% end for %}]
e={a={{a}}}"#
        )
        .unwrap();
        let mut file2 = std::fs::File::create(&file2_path)?;
        write!(file2, "List=[1,2,3]")?;

        let config: Config = Parser::from_file(&file1_path, Some(&file2_path))?;

        assert_eq!(config.a, 1);
        assert_eq!(config.b, "string");
        assert_eq!(config.c, vec![2, 3, 4]);
        assert!(config.d.contains(&2));
        assert!(config.d.contains(&4));
        assert!(config.d.contains(&6));

        assert_eq!(config.e.len(), 1);
        assert_eq!(config.e["a"], 1);
        Ok(())
    }
}
