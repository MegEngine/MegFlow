use clap::{App, Arg};
/**
 * \file flow-quickstart/main.rs
 * MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
 *
 * Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 */
use console::style;
use flow_quickstart::{error, git, git::GitConfig, info, is_git_metadata, retry, Action};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::io::{stdin, BufRead, BufReader};
use std::path::Path;
use std::sync::Mutex;
use walkdir::WalkDir;

pub struct Context {
    git_repo: String,
    project_path: String,
    custom_branch: String,
}

impl Context {
    fn new() -> Context {
        Context {
            git_repo: String::new(),
            project_path: String::new(),
            custom_branch: String::new(),
        }
    }
}

lazy_static! {
    static ref CONTEXT: Mutex<Context> = Mutex::new(Context::new());
}

// lifetime of question should not exceed Node.
// If Node destroyed, question and default &str destroyed.
struct Node<'a> {
    question: &'a str,
    default: &'a str,
    callback: fn(&str) -> Result<Action, &'static str>,
}

fn exec(commands: &[Node<'static>]) {
    let mut index = 0;
    while index < commands.len() {
        let cmd = &commands[index];
        println!("> {} [{}]", cmd.question, cmd.default);

        let mut user_input = String::new();
        stdin().read_line(&mut user_input).unwrap();

        let inp = user_input.trim();
        let result: Result<Action, &'static str>;
        if !inp.is_empty() {
            result = (cmd.callback)(inp);
        } else {
            result = (cmd.callback)(cmd.default);
        }

        match result {
            Ok(action) => match action {
                Action::Retry => continue,
                Action::Next => index += 1,
                Action::Quit => break,
            },
            Err(why) => panic!("{:?}", why),
        }
    }
}

fn make_project(s: &str) -> Result<Action, &'static str> {
    if Path::new(s).exists() {
        retry!("{} already exists, retry another path!", s);
        return Ok(Action::Retry);
    }

    CONTEXT.lock().unwrap().project_path = s.to_owned();

    Ok(Action::Next)
}

fn substitute_model_inp(s: &str) -> Result<Action, &'static str> {
    let dir = &CONTEXT.lock().unwrap().project_path;
    flow_quickstart::substitute_file_by_placeholder(dir, "lite.py", "##INP##", s)
}

fn substitute_model_path(s: &str) -> Result<Action, &'static str> {
    let dir = &CONTEXT.lock().unwrap().project_path;
    flow_quickstart::substitute_file_by_placeholder(dir, "config.toml", "##PATH##", s)
}

fn build_modelserving() -> Vec<Node<'static>> {
    let model_inp = Node {
        question: "Enter model input tensor name.",
        default: "data",
        callback: substitute_model_inp,
    };
    let model_path = Node {
        question: "Enter model fullpath.",
        default: "model.mge",
        callback: substitute_model_path,
    };
    vec![model_inp, model_path]
}

fn write_branch_name(s: &str) -> Result<Action, &'static str> {
    CONTEXT.lock().unwrap().custom_branch = s.to_string();
    Ok(Action::Next)
}

fn ask_branch() -> Vec<Node<'static>> {
    let branch_name = Node {
        question: "Enter your branch name.",
        default: "master",
        callback: write_branch_name,
    };
    vec![branch_name]
}

fn scan_all_placeholder(project_dir: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let files = WalkDir::new(project_dir)
        .sort_by_file_name()
        .contents_first(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !is_git_metadata(e))
        .filter(|e| !e.path().is_dir())
        .filter(|e| e.path() != project_dir)
        .collect::<Vec<_>>();

    // match something like ##INP##
    let re = Regex::new(r"##[_\-a-zA-Z0-9]*##").unwrap();
    for entry in files {
        let file = std::fs::File::open(&entry.path()).unwrap();
        let reader = BufReader::new(file);
        // Read the file line by line using the lines() iterator from std::io::BufRead.
        for (_, line) in reader.lines().enumerate() {
            for cap in re.captures_iter(&line.unwrap()) {
                let placeholder = cap[0].to_string();

                if map.contains_key(&placeholder) {
                    continue;
                }

                // ask input
                println!(
                    "> Enter placeholder value for {}, found in {}",
                    placeholder,
                    &entry.path().to_str().unwrap()
                );
                let mut user_input = String::new();
                stdin().read_line(&mut user_input).unwrap();

                map.insert(placeholder, user_input);
            }
        }
    }

    map
}

fn create_with_type(s: &str) -> Result<Action, &'static str> {
    const BRANCH_SERVING: &str = "template01-modelserving";
    const BRANCH_IMAGE: &str = "template02-image";
    const BRANCH_VIDEO: &str = "template03-video";

    let git_repo: String;
    {
        git_repo = CONTEXT.lock().unwrap().git_repo.clone();
    }
    let project_path: String;
    {
        project_path = CONTEXT.lock().unwrap().project_path.clone();
    }
    let path = Path::new(&project_path);

    if s.find("serving") != None {
        let config = GitConfig::new(git_repo.into(), Some(BRANCH_SERVING.to_owned()), None);
        git::create(path, config.unwrap()).unwrap();
        exec(&build_modelserving());
    } else if s.find("image") != None {
        let config = GitConfig::new(git_repo.into(), Some(BRANCH_IMAGE.to_owned()), None);
        git::create(path, config.unwrap()).unwrap();
    } else if s.find("video") != None {
        let config = GitConfig::new(git_repo.into(), Some(BRANCH_VIDEO.to_owned()), None);
        git::create(path, config.unwrap()).unwrap();
    } else if s.find("custom") != None {
        exec(&ask_branch());

        let branch_name = &CONTEXT.lock().unwrap().custom_branch;
        let config = GitConfig::new(git_repo.into(), Some(branch_name.to_owned()), None);
        match git::create(path, config.unwrap()) {
            Ok(_) => {}
            Err(why) => {
                error!("fetching branch failed, {:?}", why);
                panic!("{:?}", why);
            }
        }

        // scan all placeholder
        let placeholder_with_answer = scan_all_placeholder(path);
        return flow_quickstart::substitute_dir_by_placeholder(path, &placeholder_with_answer);
    } else {
        retry!("unrecognized command {}, retype!", s);
        return Ok(Action::Retry);
    }

    Ok(Action::Next)
}

fn start() -> Vec<Node<'static>> {
    let create_project = Node {
        question: "Enter root path for the project.",
        default: "megflow-app",
        callback: make_project,
    };
    let choose_type = Node {
        question: "Enter project type, modelserving/image/video/custom?",
        default: "modelserving",
        callback: create_with_type,
    };
    vec![create_project, choose_type]
}

fn main() {
    let matches = App::new("megflow_quickstart")
        .version("1.0.0")
        .author("megengine <megengine@megvii.com>")
        .about("interactive construct pipeline, for github user just type `megflow_quickstart`.")
        .arg(
            Arg::new("GIT")
                .short('g')
                .long("git")
                .value_name("GIT")
                .about("User-defined git repo, use https://github.com/MegEngine/MegFlow.git by default")
                .multiple_occurrences(false)
                .required(false),
        )
        .get_matches();

    if let Some(repo) = matches.value_of("GIT") {
        CONTEXT.lock().unwrap().git_repo = repo.to_owned();
    } else {
        CONTEXT.lock().unwrap().git_repo = "https://github.com/MegEngine/MegFlow.git".to_owned();
    }

    println!("{}", style("Welcome to MegFlow quickstart utility.").bold());
    println!("{}", style("Please enter values for the following settings (just press Enter to accept a default value, if one is given in brackets).").bold());

    let commands: Vec<Node<'static>> = start();
    exec(&commands);
    info!("Project created, read ${{PROJECT_dir}}/README.md to run it.");
}
