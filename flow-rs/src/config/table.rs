/**
 * \file flow-rs/src/config/table.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use toml::value::{Table, Value};

pub fn merge_table(base: Table, mut patch: Table) -> Table {
    for (k, v) in base.into_iter() {
        match try_table(v) {
            Ok(table) => {
                if let Some(patch_value) = patch.remove(&k) {
                    match try_table(patch_value) {
                        Ok(patch_table) => {
                            patch.insert(k, Value::Table(merge_table(table, patch_table)));
                        }
                        Err(patch_value) => {
                            patch.insert(k, patch_value);
                        }
                    }
                } else {
                    patch.insert(k, Value::Table(table));
                }
            }
            Err(value) => {
                let entry = patch.entry(k);
                entry.or_insert(value);
            }
        }
    }
    patch
}

fn try_table(value: Value) -> Result<Table, Value> {
    match value {
        Value::Table(table) => Ok(table),
        _ => Err(value),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use toml::toml;

    fn table(value: Value) -> Table {
        match value {
            Value::Table(table) => table,
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_merge() {
        let base = toml! {
            a = 1
            b = [1, 2, 3]

            [c]
            d = 1
            e = [1, 2, 3]
        };

        let patch = toml! {
            f = 2
            b = [3, 4]

            [c]
            g = 2
            d = 3
        };

        let base = table(base);
        let patch = table(patch);

        let merge = merge_table(base, patch);

        assert_eq!(
            merge,
            table(toml! {
                a = 1
                f = 2
                b = [3, 4]

                [c]
                d = 3
                g = 2
                e = [1, 2, 3]
            })
        );
    }
}
