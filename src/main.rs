use chrono::Utc;
use colored::*;
use cpm::codeforces::CodeforcesParser;
use cpm::parser::Parser;
use cpm::util;
use cpm::{atcoder::AtCoderParser, util::ProblemInfo};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Read},
};

enum SubCommand {
    Init,
    Get,
    Open,
    Download,
    Login,
    Root,
    List,
    Test,
}
impl SubCommand {
    fn value(&self) -> String {
        match *self {
            SubCommand::Init => "init".to_string(),
            SubCommand::Get => "get".to_string(),
            SubCommand::Open => "open".to_string(),
            SubCommand::Download => "download".to_string(),
            SubCommand::Login => "login".to_string(),
            SubCommand::Root => "root".to_string(),
            SubCommand::List => "list".to_string(),
            SubCommand::Test => "test".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    root: String,
}

fn init_config() -> Result<(), anyhow::Error> {
    std::fs::create_dir_all(dirs::home_dir().unwrap().join(".config").join("cpm"))?;

    let config_file = dirs::home_dir()
        .unwrap()
        .join(".config")
        .join("cpm")
        .join("config.json");
    if !config_file.exists() {
        let config = Config {
            root: "".to_string(),
        };
        serde_json::to_writer(&std::fs::File::create(config_file.clone())?, &config)?;
    }
    let fallback_cmd = if cfg!(target_os = "linux") {
        "xdg-open".to_string()
    } else if cfg!(target_os = "macos") {
        "open".to_string()
    } else if cfg!(target_os = "windows") {
        let cmd = std::path::Path::new(&std::env::var("SYSTEMROOT").unwrap())
            .join("System32")
            .join("rundll32.exe");
        cmd.to_str().unwrap().to_string()
    } else {
        unreachable!("UNKNOWN OS");
    };
    let open_cmd = std::env::var("EDITOR").unwrap_or(fallback_cmd);
    std::process::Command::new(open_cmd)
        .arg(&config_file)
        .status()?;

    Ok(())
}
fn load_config() -> Result<Config, anyhow::Error> {
    let config_file = dirs::home_dir()
        .unwrap()
        .join(".config")
        .join("cpm")
        .join("config.json");

    let file = std::fs::File::open(config_file)?;
    let reader = std::io::BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    Ok(config)
}
struct Cpm {
    client: reqwest::Client,
    //for request
    cookie_headers: HeaderMap,
    //from response
    html: Option<String>,
}

impl Cpm {
    fn new() -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        Cpm {
            client,
            cookie_headers: HeaderMap::new(),
            html: None,
        }
    }

    fn create_problem_dir<T: Parser>(
        &self,
        url: &url::Url,
        parser: &T,
        sample_verbose: bool,
    ) -> Result<(), anyhow::Error> {
        let host_name = url.host_str().unwrap();
        let config = load_config()?;
        let path = {
            let mut path = std::path::PathBuf::from(config.root).join(host_name);
            std::path::PathBuf::from(url.path())
                .iter()
                .filter_map(|comp| {
                    let comp = comp.to_str().unwrap();
                    if comp != std::path::MAIN_SEPARATOR.to_string() {
                        Some(comp)
                    } else {
                        None
                    }
                })
                .for_each(|name| {
                    path.push(name);
                });
            path
        };
        let samples = parser.sample_cases();

        if sample_verbose {
            println!("====== Download Result ======");
        }
        util::create_sample_test_files(&samples, path.join("sample").to_str())?;
        for (idx, (input, output)) in samples.iter().enumerate() {
            if sample_verbose {
                println!("=== Sample Test Case {} ===", idx + 1);
                println!("Input:\n{}\nOutput:\n{}", input, output);
            }
        }
        if sample_verbose {
            println!("=============================");
        }

        let info = ProblemInfo {
            url: url.to_string(),
            contest_name: parser.contest_name().expect("failed to get contest name"),
            problem_name: parser.problem_name().expect("failed to get problem name"),
            created_at: Some(Utc::now()),
        };
        util::create_problem_info_json(info, &path)?;
        println!(
            "Created a directory and saved sample cases: {}",
            path.to_str().unwrap()
        );
        Ok(())
    }
    pub fn init(&self) -> Result<(), anyhow::Error> {
        init_config()?;
        Ok(())
    }
    pub async fn get(&mut self, url: &str) -> Result<(), anyhow::Error> {
        let url = url::Url::parse(url)?;

        let host = url.host_str();
        match host {
            Some("atcoder.jp") => {
                if let Ok(cookie_headers) = util::local_cookie_headers() {
                    self.cookie_headers = cookie_headers
                }
                let mut paths: Vec<_> = url.path().split('/').collect();
                if !paths.contains(&"tasks") {
                    paths.push("tasks");
                }
                let path = paths.join("/");
                let mut url = url;
                url.set_path(&path);

                let resp = self.call_get_request(url.as_str()).await?;
                self.parse_response(resp).await?;
                let parser = AtCoderParser::new(self.html.as_ref().unwrap());

                let query = url.path().split('/').last().expect("No element");

                match query {
                    "tasks" => {
                        if let Some(url_list) = parser.problem_url_list() {
                            for task_url in url_list.iter() {
                                let task_url = url.join(task_url)?;
                                let resp = self.call_get_request(task_url.as_str()).await?;
                                self.parse_response(resp).await?;
                                let parser = AtCoderParser::new(self.html.as_ref().unwrap());
                                self.create_problem_dir(&task_url, &parser, false)?;
                            }
                        }
                    }
                    _ => {
                        self.create_problem_dir(&url, &parser, true)?;
                    }
                }
            }
            Some("codeforces.com") => {
                let resp = self.call_get_request(url.as_str()).await?;
                self.parse_response(resp).await?;
                let parser = CodeforcesParser::new(self.html.as_ref().unwrap());
                let problem_url_list = parser.problem_url_list(url.path());

                if !problem_url_list.is_empty() {
                    for task_url in problem_url_list.iter() {
                        let task_url = url.join(task_url)?;
                        let resp = self.call_get_request(task_url.as_str()).await?;
                        self.parse_response(resp).await?;
                        let parser = CodeforcesParser::new(self.html.as_ref().unwrap());
                        self.create_problem_dir(&task_url, &parser, false)?;
                    }
                } else {
                    self.create_problem_dir(&url, &parser, true)?;
                }
            }
            Some(host) => {
                println!("{} isn't supported yet. X(", host);
            }
            _ => {
                println!("Something wrong happened");
            }
        }
        Ok(())
    }
    pub async fn download(&mut self, url: &str) -> Result<(), anyhow::Error> {
        let url = url::Url::parse(url)?;
        if let Ok(cookie_headers) = util::local_cookie_headers() {
            self.cookie_headers = cookie_headers;
        }
        let resp = self.call_get_request(url.as_str()).await?;
        self.parse_response(resp).await?;

        let parser = AtCoderParser::new(self.html.as_ref().unwrap());
        let sample_test_cases = parser.sample_cases();

        println!("====== Download Result ======");

        util::create_sample_test_files(&sample_test_cases, None)?;
        for (idx, (input, output)) in sample_test_cases.iter().enumerate() {
            println!("=== Sample Test Case {} ===", idx + 1);
            println!("Input:\n{}\nOutput:\n{}", input, output);
        }

        println!("=============================");
        Ok(())
    }
    pub fn open(&self) -> Result<(), anyhow::Error> {
        let file = std::fs::File::open(".problem.json")?;
        let reader = BufReader::new(file);
        let info: ProblemInfo = serde_json::from_reader(reader)?;
        webbrowser::open(&info.url)?;
        Ok(())
    }
    pub fn root(&self) -> Result<(), anyhow::Error> {
        let config = load_config()?;
        println!("{}", config.root);
        Ok(())
    }
    pub fn list(&self, all: bool, recent: bool) -> Result<(), anyhow::Error> {
        let config = load_config()?;
        let now = Utc::now();
        for (parent, entry) in walkdir::WalkDir::new(config.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|dir| {
                let file_name = dir.file_name().to_str().unwrap();
                file_name == ".problem" || file_name == ".problem.json"
            })
            .filter_map(|entry| {
                entry
                    .path()
                    .parent()
                    .map(|dir| (dir.to_string_lossy().to_string(), entry.clone()))
            })
        {
            let reader = BufReader::new(File::open(entry.path())?);
            let info = serde_json::from_reader::<_, ProblemInfo>(reader);

            if let Ok(info) = info {
                let contest_name = info
                    .contest_name
                    .chars()
                    .filter(|&c| c != '\n' && c != '\t')
                    .collect::<String>();
                let problem_name = info
                    .problem_name
                    .chars()
                    .filter(|&c| c != '\n' && c != '\t')
                    .collect::<String>();

                // An old format doesn't support `created_at`. Skip it
                if recent
                    && info
                        .created_at
                        .map_or(true, |created_at| (now - created_at).num_hours() >= 24)
                {
                    continue;
                }

                if all {
                    println!("{} {} {}", contest_name, problem_name, parent);
                } else {
                    println!("{}", parent);
                }
            } else {
                // An old format doesn't support `created_at`. Skip it
                if recent {
                    continue;
                }
                println!("{}", parent);
            }
        }
        Ok(())
    }
    pub fn test(&self, command: &str) -> Result<(), anyhow::Error> {
        //current dir is problem?
        let mut sample_case_paths = vec![]; //(input, output)
        if std::path::Path::new(".problem.json").exists()
            || std::path::Path::new(".problem").exists()
        {
            let mut case = 1;
            loop {
                let input = format!("sample/sample_input_{}.txt", case);
                let output = format!("sample/sample_output_{}.txt", case);
                let input_file_path = std::path::PathBuf::from(&input);
                let output_file_path = std::path::PathBuf::from(&output);
                if input_file_path.exists() && output_file_path.exists() {
                    sample_case_paths.push((input_file_path.clone(), output_file_path.clone()));
                    case += 1;
                    continue;
                }
                //support old format
                let old_input = format!("sample/sample_{:02}_in.txt", case - 1);
                let old_output = format!("sample/sample_{:02}_out.txt", case - 1);
                println!("{} {}", old_input, old_output);
                let input_file_path = std::path::PathBuf::from(&old_input);
                let output_file_path = std::path::PathBuf::from(&old_output);
                if input_file_path.exists() && output_file_path.exists() {
                    sample_case_paths.push((input_file_path.clone(), output_file_path.clone()));
                    case += 1;
                    continue;
                }
                break;
            }
        }
        sample_case_paths.sort();
        println!("RUNNING TEST CASES...");
        let mut ac_cnt = 0;
        for (input_file_path, output_file_path) in sample_case_paths.iter() {
            println!("-----------------------------------------");
            let input_file = std::fs::File::open(input_file_path)?;
            let start = std::time::Instant::now();
            let commands: Vec<&str> = command.split_whitespace().collect();
            let command = <&str>::clone(commands.first().expect("No command"));
            let args: Vec<&str> = commands.into_iter().skip(1).collect();
            let command_output_child = std::process::Command::new(command)
                .stdin(input_file)
                .stdout(std::process::Stdio::piped())
                .args(args)
                .spawn()?;

            let output = command_output_child.wait_with_output()?;
            let elapsed = start.elapsed();
            let output_string = String::from_utf8(output.stdout).map(|s| s.trim().to_string())?;

            println!(
                "Input: {}",
                input_file_path.file_name().unwrap().to_str().unwrap()
            );
            println!(
                "Output: {}",
                output_file_path.file_name().unwrap().to_str().unwrap()
            );
            let mut sample_output_string = String::new();
            std::fs::File::open(output_file_path)?.read_to_string(&mut sample_output_string)?;
            println!("{} {} ms", "[TIME]".cyan(), elapsed.as_millis());

            let mut ok = true;

            let mut output_iter = output_string.lines();

            for s in sample_output_string.lines() {
                if let Some(o) = output_iter.next() {
                    if o.trim() != s.trim() {
                        ok = false;
                    }
                } else {
                    ok = false;
                }
            }
            for rest in output_iter {
                ok &= rest.chars().all(|c| c.is_whitespace() || c == '\n');
            }

            if ok {
                println!("{}", "[OK]".green());
                ac_cnt += 1;
            } else {
                println!("{}", "[Wrong Answer]".yellow());
                //diff
                println!("The output is");
                println!("{}", output_string);
                println!("The judge is");
                println!("{}", sample_output_string);
            }
        }
        let status = if ac_cnt == sample_case_paths.len() {
            format!(
                "{} : {} / {}",
                "[Accept]".green(),
                ac_cnt,
                sample_case_paths.len()
            )
        } else {
            format!(
                "{} : {} / {}",
                "[Wrong Answer]".yellow(),
                ac_cnt,
                sample_case_paths.len()
            )
        };
        println!("{}", status);

        Ok(())
    }

    pub async fn login(&mut self, url: &str) -> Result<(), anyhow::Error> {
        let url = url::Url::parse(url)?;
        let resp = self.call_get_request(url.as_str()).await?;
        self.parse_response(resp).await?;
        let parser = AtCoderParser::new(self.html.as_ref().unwrap());
        //necessary information and parameters to login AtCoder
        let csrf_token = parser.csrf_token().unwrap();
        let (username, password) = util::username_and_password();
        let params = {
            let mut params = std::collections::HashMap::new();
            params.insert("username", username);
            params.insert("password", password);
            params.insert("csrf_token", csrf_token);
            params
        };
        //make a post request and try to login
        let resp = self.call_post_request(url.as_str(), &params).await?;
        //save your cookie in your local
        util::save_cookie_in_local(&resp)?;
        Ok(())
    }

    async fn call_post_request(
        &self,
        url: &str,
        params: &std::collections::HashMap<&str, String>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let resp = self
            .client
            .post(url)
            .headers(self.cookie_headers.clone())
            .form(params)
            .send()
            .await?;
        Ok(resp)
    }
    async fn call_get_request(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let resp = self
            .client
            .get(url)
            .headers(self.cookie_headers.clone())
            .send()
            .await?;
        Ok(resp)
    }

    async fn parse_response(&mut self, response: reqwest::Response) -> Result<(), anyhow::Error> {
        //cookie
        let mut cookie_headers = HeaderMap::new();
        response.cookies().for_each(|cookie| {
            cookie_headers.insert(
                COOKIE,
                HeaderValue::from_str(&format!("{}={}", cookie.name(), cookie.value())).unwrap(),
            );
        });
        self.cookie_headers = cookie_headers;
        self.html = Some(response.text().await?);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap::App::new("cpm")
        .version("1.1")
        .author("Hitoshi Togasaki. <togasakitogatoga+github.com>")
        .about(
            "Download sample test cases of AtCoder problem

Example:
    //Get
    cpm get https://atcoder.jp/contests/abc154/tasks
    cpm get https://atcoder.jp/contests/abc154/tasks/abc154_a

    //Download
    cpm download https://atcoder.jp/contests/agc035/tasks/agc035_a

    //Login
    cpm login",
        )
        .subcommand(
            clap::SubCommand::with_name(&SubCommand::Init.value()).about("Initialize config file"),
        )
        .subcommand(
            clap::SubCommand::with_name(&SubCommand::Get.value())
                .about("Create a new directory from URL under root path")
                .arg(
                    clap::Arg::with_name("url")
                        .help("A URL of problem")
                        .required(true),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name(&SubCommand::Open.value()).about("Open the problem page"),
        )
        .subcommand(
            clap::SubCommand::with_name(&SubCommand::Download.value())
                .about("Download sample test cases in your local")
                .arg(
                    clap::Arg::with_name("url")
                        .help("A URL of AtCoder problem")
                        .required(true),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name(&SubCommand::Login.value())
                .about("Login AtCoder and save session in your local")
                .arg(clap::Arg::with_name("url").help("A login URL of AtCoder")),
        )
        .subcommand(clap::SubCommand::with_name(&SubCommand::Root.value()).about("Show root path"))
        .subcommand(
            clap::SubCommand::with_name(&SubCommand::List.value())
                .about("List local directories under root path")
                .arg_from_usage("-a, --all 'Print problem's information(contest name, problem name, directory).")
                .arg_from_usage(
                    "-r, --recent 'Print only recent problems (less than 24 hours).'",
                ),
        )
        .subcommand(
            clap::SubCommand::with_name(&SubCommand::Test.value())
                .about("Test sample test cases")
                .arg(
                    clap::Arg::with_name("command")
                        .help("An execute command run for test cases")
                        .required(true),
                ),
        )
        .get_matches();
    //run sub commands
    let mut cpm = Cpm::new();
    if matches
        .subcommand_matches(&SubCommand::Init.value())
        .is_some()
    {
        match cpm.init() {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
    if matches
        .subcommand_matches(&SubCommand::Open.value())
        .is_some()
    {
        match cpm.open() {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1)
            }
        }
    }

    if matches
        .subcommand_matches(&SubCommand::Root.value())
        .is_some()
    {
        match cpm.root() {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1)
            }
        }
    }

    if let Some(matched) = matches.subcommand_matches(&SubCommand::Get.value()) {
        match cpm.get(matched.value_of("url").unwrap()).await {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(matched) = matches.subcommand_matches(&SubCommand::Download.value()) {
        match cpm.download(matched.value_of("url").unwrap()).await {
            Ok(_) => {
                std::process::exit(0);
            }
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
    if let Some(matched) = matches.subcommand_matches(&SubCommand::Login.value()) {
        match cpm
            .login(
                matched
                    .value_of("url")
                    .unwrap_or("https://atcoder.jp/login"),
            )
            .await
        {
            Ok(_) => {
                std::process::exit(0);
            }
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(args) = matches.subcommand_matches(&SubCommand::List.value()) {
        match cpm.list(args.is_present("all"), args.is_present("recent")) {
            Ok(_) => {
                std::process::exit(0);
            }
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
    if let Some(matched) = matches.subcommand_matches(&SubCommand::Test.value()) {
        match cpm.test(matched.value_of("command").unwrap()) {
            Ok(_) => {
                std::process::exit(0);
            }
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
