mod util;

use colored::*;
use itertools::Itertools;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use selectors::Element;
use serde::{Deserialize, Serialize};
use std::io::Read;
use util::ProblemInfo;

enum SubCommand {
    Get,
    Download,
    Login,
    Root,
    List,
    Test,
}
impl SubCommand {
    fn value(&self) -> String {
        match *self {
            SubCommand::Get => "get".to_string(),
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
fn load_config() -> Result<Config, failure::Error> {
    std::fs::create_dir_all(dirs::home_dir().unwrap().join(".config").join("cpm"))?;
    let config_file = dirs::home_dir()
        .unwrap()
        .join(".config")
        .join("cpm")
        .join("config.json");
    if !config_file.exists() {}

    let file = std::fs::File::open(config_file)?;
    let reader = std::io::BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    Ok(config)
}
struct AtCoder {
    client: reqwest::Client,
    //for request
    cookie_headers: HeaderMap,
    //from response
    html: Option<String>,
}

struct AtCoderParser {
    document: scraper::Html,
}

impl AtCoderParser {
    fn new(html: &str) -> AtCoderParser {
        AtCoderParser {
            document: scraper::Html::parse_document(html),
        }
    }
    fn problem_name(&self) -> Option<String> {
        let main_container_selector =
            scraper::Selector::parse(r#"div[id="main-container"]"#).unwrap();

        if let Some(main_container) = self.document.select(&main_container_selector).next() {
            if let Some(col_selector) = main_container
                .select(&scraper::Selector::parse(r#"div[class="col-sm-12"]"#).unwrap())
                .next()
            {
                if let Some(span_selector) = col_selector
                    .select(&scraper::Selector::parse(r#"span[class="h2"]"#).unwrap())
                    .next()
                {
                    let problem_name = span_selector.text().collect::<String>();
                    return Some(problem_name);
                }
            }
        }
        None
    }
    fn contest_name(&self) -> Option<String> {
        let contest_title_selector =
            scraper::Selector::parse(r#"a[class="contest-title"]"#).unwrap();
        if let Some(contest_title_selector) = self.document.select(&contest_title_selector).next() {
            let contest_name = contest_title_selector.text().collect::<String>();
            return Some(contest_name);
        }
        None
    }
    fn sample_cases(&self) -> Option<Vec<(String, String)>> {
        let task_statement_selector =
            scraper::Selector::parse(r#"div[id="task-statement"]"#).unwrap();
        let pre_selector = scraper::Selector::parse("pre").unwrap();
        let h3_selector = scraper::Selector::parse("h3").unwrap();
        let input_h3_text = vec!["入力例", "Sample Input"];
        let output_h3_text = vec!["出力例", "Sample Output"];

        let mut input_cases = vec![];
        let mut output_cases = vec![];
        if let Some(task_statement) = self.document.select(&task_statement_selector).next() {
            for pre in task_statement.select(&pre_selector) {
                if let Some(pre_parent) = pre.parent_element() {
                    if let Some(h3) = pre_parent.select(&h3_selector).next() {
                        let h3_text = h3.text().collect::<String>();
                        let input = input_h3_text.iter().any(|&x| h3_text.contains(x));
                        let output = output_h3_text.iter().any(|&x| h3_text.contains(x));
                        let text = pre.text().collect::<String>();
                        if input {
                            input_cases.push(text);
                        } else if output {
                            output_cases.push(text);
                        }
                    }
                }
            }
        } else {
            return None;
        }
        // make cases unique to remove extra duplicated language cases
        let input_cases: Vec<String> = input_cases.into_iter().unique().collect();
        let output_cases: Vec<String> = output_cases.into_iter().unique().collect();
        let sample_test_cases: Vec<(String, String)> = input_cases
            .into_iter()
            .zip(output_cases)
            .map(|(input, output)| (input, output))
            .collect();
        Some(sample_test_cases)
    }

    fn csrf_token(&self) -> Option<String> {
        let selector = scraper::Selector::parse(r#"input[name="csrf_token"]"#).unwrap();
        if let Some(element) = self.document.select(&selector).next() {
            if let Some(token) = element.value().attr("value") {
                return Some(token.to_string());
            }
        }
        None
    }
}

impl AtCoder {
    fn new() -> AtCoder {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        AtCoder {
            client: client,
            cookie_headers: HeaderMap::new(),
            html: None,
        }
    }
    pub async fn get(&mut self, url: &str) -> Result<(), failure::Error> {
        let url = url::Url::parse(url)?;
        if let Ok(cookie_headers) = util::local_cookie_headers() {
            self.cookie_headers = cookie_headers
        }
        let resp = self.call_get_request(url.as_str()).await?;
        self.parse_response(resp).await?;
        let parser = AtCoderParser::new(&self.html.as_ref().unwrap());

        let mut problem_name = parser.problem_name().unwrap();
        let mut contest_name = parser.contest_name().unwrap();
        //Remove extra whitespace
        problem_name.retain(|x| !x.is_whitespace());
        contest_name.retain(|x| !x.is_whitespace());
        println!("{} {}", contest_name, problem_name);
        let config = load_config()?;
        let path = std::path::PathBuf::from(config.root)
            .join("atcoder.jp")
            .join(contest_name)
            .join(problem_name);
        let sample_test_cases = parser.sample_cases();
        println!("====== Download Result ======");
        if let Some(samples) = sample_test_cases {
            util::create_sample_test_files(&samples, path.join("sample").to_str())?;
            for (idx, (input, output)) in samples.iter().enumerate() {
                println!("=== Sample Test Case {} ===", idx + 1);
                println!("Input:\n{}\nOutput:\n{}", input, output);
            }
        }
        println!("=============================");
        let info = ProblemInfo {
            url: url.to_string(),
            contest_name: parser.contest_name().unwrap(),
            problem_name: parser.problem_name().unwrap(),
        };
        util::create_problem_info_json(info, &path)?;
        println!("{}", path.to_str().unwrap());
        Ok(())
    }
    pub async fn download(&mut self, url: &str) -> Result<(), failure::Error> {
        let url = url::Url::parse(url)?;
        if let Ok(cookie_headers) = util::local_cookie_headers() {
            self.cookie_headers = cookie_headers;
        }
        let resp = self.call_get_request(url.as_str()).await?;
        self.parse_response(resp).await?;

        let parser = AtCoderParser::new(&self.html.as_ref().unwrap());
        let sample_test_cases = parser.sample_cases();

        println!("====== Download Result ======");
        if let Some(samples) = sample_test_cases {
            util::create_sample_test_files(&samples, None)?;
            for (idx, (input, output)) in samples.iter().enumerate() {
                println!("=== Sample Test Case {} ===", idx + 1);
                println!("Input:\n{}\nOutput:\n{}", input, output);
            }
        }
        println!("=============================");
        Ok(())
    }
    pub fn list(&self) -> Result<(), failure::Error> {
        let config = load_config()?;
        for entry in walkdir::WalkDir::new(config.root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let file_name = entry.file_name().to_str().unwrap();
            if file_name == ".problem" || file_name == ".problem.json" {
                if let Some(dir) = entry.path().parent() {
                    println!("{}", dir.to_str().unwrap());
                }
            }
        }
        Ok(())
    }
    pub fn test(&self, command: &str) -> Result<(), failure::Error> {
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
        for (input_file_path, output_file_path) in sample_case_paths.iter() {
            println!("-----------------------------------------");
            let input_file = std::fs::File::open(input_file_path)?;
            let start = std::time::Instant::now();
            let command_output_child = std::process::Command::new(command)
                .stdin(input_file)
                .stdout(std::process::Stdio::piped())
                .arg(input_file_path)
                .spawn()?;
            let output = command_output_child.wait_with_output()?;
            let elapsed = start.elapsed();
            let output_string = String::from_utf8(output.stdout).unwrap();
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
            if output_string == sample_output_string {
                println!("{}", "[OK]".green());
            } else {
                println!("{}", "[Wrong Answer]".yellow());
                //diff
                println!("The output is");
                println!("{}", output_string);
                println!("The judge is");
                println!("{}", sample_output_string);
            }
        }

        Ok(())
    }

    pub async fn login(&mut self, url: &str) -> Result<(), failure::Error> {
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

    async fn parse_response(&mut self, response: reqwest::Response) -> Result<(), failure::Error> {
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
        .version("1.0")
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
            clap::SubCommand::with_name(&SubCommand::Get.value())
                .about("Create a new directory from URL under root path")
                .arg(
                    clap::Arg::with_name("url")
                        .help("A URL of problem")
                        .required(true),
                ),
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
                .about("List local directories under root path"),
        )
        .subcommand(
            clap::SubCommand::with_name(&SubCommand::Test.value())
                .about("Test sample test cases")
                .arg(
                    clap::Arg::with_name("command")
                        .help("A execute command run for test cases")
                        .required(true),
                ),
        )
        .get_matches();
    //run sub commands
    let mut atcoder = AtCoder::new();
    if let Some(ref matched) = matches.subcommand_matches(&SubCommand::Get.value()) {
        match atcoder.get(matched.value_of("url").unwrap()).await {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref matched) = matches.subcommand_matches(&SubCommand::Download.value()) {
        match atcoder.download(matched.value_of("url").unwrap()).await {
            Ok(_) => {
                std::process::exit(0);
            }
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
    if let Some(ref matched) = matches.subcommand_matches(&SubCommand::Login.value()) {
        match atcoder
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
    if let Some(_) = matches.subcommand_matches(&SubCommand::List.value()) {
        match atcoder.list() {
            Ok(_) => {
                std::process::exit(0);
            }
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1);
            }
        }
    }
    if let Some(ref matched) = matches.subcommand_matches(&SubCommand::Test.value()) {
        match atcoder.test(matched.value_of("command").unwrap()) {
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
