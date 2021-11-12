/**
 * \file flow-quickstart/log.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use console::Emoji;

pub static ERROR: Emoji<'_, '_> = Emoji("⛔  ", "");
pub static SPARKLE: Emoji<'_, '_> = Emoji("✨  ", "");
pub static WARN: Emoji<'_, '_> = Emoji("⚠️  ", "");
pub static WRENCH: Emoji<'_, '_> = Emoji("🔧  ", "");
pub static INFO: Emoji<'_, '_> = Emoji("💡  ", "");

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        println!("{} {}",
            $crate::log::WARN,
            format!($($arg)*)
        );
    })
}

#[macro_export]
macro_rules! retry {
    ($($arg:tt)*) => ({
        println!("{} {}",
            $crate::log::WRENCH,
            format!($($arg)*)
        );
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        println!("{} {}",
            $crate::log::INFO,
            format!($($arg)*)
        );
    })
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        println!("{} {}",
            $crate::log::ERROR,
            format!($($arg)*)
        );
    })
}
