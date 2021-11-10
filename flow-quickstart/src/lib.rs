/**
 * \file flow-quickstart/lib.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
pub mod git;
pub mod log;

use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub enum Action {
    Retry,
    Next,
    Quit,
}

pub fn is_git_metadata(entry: &DirEntry) -> bool {
    entry
        .path()
        .components()
        .any(|c| c == std::path::Component::Normal(".git".as_ref()))
}

pub fn substitute_file_by_placeholder(
    dir: &str,
    filename: &str,
    from: &str,
    to: &str,
) -> Result<Action, &'static str> {
    let pp = Path::new(dir).join(filename);
    if !pp.exists() {
        error!(
            "critical file {} not exists in the template branch",
            filename
        );

        return Ok(Action::Quit);
    }

    let mut contents: Vec<String> = Vec::new();

    {
        let fromfile = std::fs::File::open(&pp).unwrap();
        let reader = BufReader::new(fromfile);
        // Read the file line by line using the lines() iterator from std::io::BufRead.
        for (_, line) in reader.lines().enumerate() {
            let ll = line.unwrap();

            if ll.find(from) != None {
                contents.push(ll.replace(from, to).to_owned());
            } else {
                contents.push(ll.to_owned());
            }
        }
    }

    let mut tofile = fs::File::create(&pp).expect("create file failed");
    for line in contents {
        tofile.write_all(line.as_bytes()).expect("write failed");
        tofile.write_all("\n".as_bytes()).expect("write failed");
    }

    Ok(Action::Next)
}

pub fn substitute_dir_by_placeholder(
    project_dir: &Path,
    map: &HashMap<String, String>,
) -> Result<Action, &'static str> {
    let files = WalkDir::new(project_dir)
        .sort_by_file_name()
        .contents_first(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !is_git_metadata(e))
        .filter(|e| !e.path().is_dir())
        .filter(|e| e.path() != project_dir)
        .collect::<Vec<_>>();

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-"),
    );

    for (_, entry) in files.into_iter().enumerate() {
        let file = std::fs::File::open(&entry.path()).unwrap();
        let reader = BufReader::new(file);

        let mut contents: Vec<String> = Vec::new();
        // Read the file line by line using the lines() iterator from std::io::BufRead.
        for (_, line) in reader.lines().enumerate() {
            let mut ll = line.unwrap().to_owned();

            for (key, value) in map {
                if ll.find(key) != None {
                    ll = ll.replace(key, value).to_owned();
                }
            }
            contents.push(ll);
        }

        let mut tofile = fs::File::create(&entry.path()).expect("create file failed");
        for line in contents {
            tofile.write_all(line.as_bytes()).expect("write failed");
            tofile.write_all("\n".as_bytes()).expect("write failed");
        }
        pb.inc(1);
    }
    pb.finish();

    Ok(Action::Next)
}
