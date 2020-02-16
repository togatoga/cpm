extern crate clap;
extern crate cookie;
extern crate dirs;
extern crate failure;
extern crate reqwest;
extern crate rpassword;
extern crate scraper;
extern crate selectors;
extern crate tokio;
extern crate url;

use itertools::Itertools;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use selectors::Element;
use std::io::{BufRead, Write};

enum SubCommand {
    Get,
    Download,
    Login,
}
impl SubCommand {
    fn value(&self) -> String {
        match *self {
            SubCommand::Get => "get".to_string(),
            SubCommand::Download => "download".to_string(),
            SubCommand::Login => "login".to_string(),
        }
    }
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
        if let Ok(cookie_headers) = AtCoder::local_cookie_headers() {
            self.cookie_headers = cookie_headers
        }
        let resp = self.call_get_request(url.as_str()).await?;
        self.parse_response(resp).await?;
        let parser = AtCoderParser::new(&self.html.as_ref().unwrap());

        let problem_name = parser.problem_name().unwrap();
        let contest_name = parser.contest_name().unwrap();
        let sample_test_cases = parser.sample_cases();
        println!("{} {}", problem_name, contest_name);
        println!("====== Download Result ======");
        if let Some(samples) = sample_test_cases {
            AtCoder::create_sample_test_files(&samples)?;
            for (idx, (input, output)) in samples.iter().enumerate() {
                println!("=== Sample Test Case {} ===", idx + 1);
                println!("Input:\n{}\nOutput:\n{}", input, output);
            }
        }
        println!("=============================");
        Ok(())
    }
    pub async fn download(&mut self, url: &str) -> Result<(), failure::Error> {
        let url = url::Url::parse(url)?;
        if let Ok(cookie_headers) = AtCoder::local_cookie_headers() {
            self.cookie_headers = cookie_headers;
        }
        let resp = self.call_get_request(url.as_str()).await?;
        self.parse_response(resp).await?;

        let parser = AtCoderParser::new(&self.html.as_ref().unwrap());
        let sample_test_cases = parser.sample_cases();

        println!("====== Download Result ======");
        if let Some(samples) = sample_test_cases {
            AtCoder::create_sample_test_files(&samples)?;
            for (idx, (input, output)) in samples.iter().enumerate() {
                println!("=== Sample Test Case {} ===", idx + 1);
                println!("Input:\n{}\nOutput:\n{}", input, output);
            }
        }
        println!("=============================");
        Ok(())
    }
    pub async fn login(&mut self, url: &str) -> Result<(), failure::Error> {
        let url = url::Url::parse(url)?;
        let resp = self.call_get_request(url.as_str()).await?;
        self.parse_response(resp).await?;
        let parser = AtCoderParser::new(self.html.as_ref().unwrap());
        //necessary information and parameters to login AtCoder
        let csrf_token = parser.csrf_token().unwrap();
        let (username, password) = AtCoder::username_and_password();
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
        AtCoder::save_cookie_in_local(&resp)?;
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

    //utils
    fn create_sample_test_files(test_cases: &[(String, String)]) -> Result<(), failure::Error> {
        for (idx, (input, output)) in test_cases.iter().enumerate() {
            //e.g sample_input_1.txt sample_output_1.txt
            let input_file_name = format!("sample_input_{}.txt", idx + 1);

            let mut input_file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(input_file_name)?;
            input_file.write_all(input.as_bytes())?;

            let output_file_name = format!("sample_output_{}.txt", idx + 1);
            let mut output_file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(output_file_name)?;
            output_file.write_all(output.as_bytes())?;
        }
        Ok(())
    }
    fn save_cookie_in_local(response: &reqwest::Response) -> Result<(), failure::Error> {
        let cookies_str = response
            .cookies()
            .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
            .join(";");
        let path = dirs::home_dir().unwrap().join(".atcoder-sample-downloader");
        //create $HOME/.atcoder-sample-downloader
        std::fs::create_dir_all(path.clone())?;
        //create cookie.jar under this directory
        let cookie_path = path.join("cookie.jar");
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(cookie_path.clone())?
            .write_all(cookies_str.as_bytes())?;
        println!("SAVED YOUR COOKIE IN {}", cookie_path.to_str().unwrap());
        Ok(())
    }
    fn username_and_password() -> (String, String) {
        println!("Please input Your username and password");
        let username = rpassword::read_password_from_tty(Some("Username > ")).unwrap();
        let password = rpassword::read_password_from_tty(Some("Password > ")).unwrap();
        (username, password)
    }
    fn local_cookie_headers() -> Result<HeaderMap, failure::Error> {
        let cookiejar_path = dirs::home_dir()
            .unwrap()
            .join(".atcoder-sample-downloader")
            .join("cookie.jar");
        let file = std::fs::File::open(cookiejar_path)?;
        let reader = std::io::BufReader::new(file);

        let mut cookie_headers = HeaderMap::new();
        reader.lines().for_each(|line| {
            cookie_headers.insert(
                COOKIE,
                HeaderValue::from_str(&format!("{}", line.unwrap())).unwrap(),
            );
        });
        Ok(cookie_headers)
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
                .about("Create a new directory from URL")
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
    Ok(())
}
