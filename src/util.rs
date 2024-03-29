use chrono::serde::ts_seconds_option;
use chrono::Utc;
use itertools::Itertools;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, Write},
    path::Path,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct ProblemInfo {
    pub url: String,
    pub contest_name: String,
    pub problem_name: String,
    #[serde(with = "ts_seconds_option")]
    pub created_at: Option<chrono::DateTime<Utc>>,
}

pub fn create_problem_info_json(info: ProblemInfo, path: &Path) -> Result<(), anyhow::Error> {
    let mut json_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path.join(".problem.json"))?;
    json_file.write_all(serde_json::to_string(&info).unwrap().as_bytes())?;
    Ok(())
}
pub fn create_sample_test_files(
    test_cases: &[(String, String)],
    path: Option<&str>,
) -> Result<(), anyhow::Error> {
    let root_path = if let Some(p) = path {
        std::path::PathBuf::from(p)
    } else {
        std::env::current_dir()?
    };

    std::fs::create_dir_all(root_path.clone())?;
    for (idx, (input, output)) in test_cases.iter().enumerate() {
        //e.g sample_input_1.txt sample_output_1.txt
        let input_file_name = format!("sample_input_{}.txt", idx + 1);
        let mut input_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(root_path.join(input_file_name))?;
        input_file.write_all(input.as_bytes())?;

        let output_file_name = format!("sample_output_{}.txt", idx + 1);
        let mut output_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(root_path.join(output_file_name))?;
        output_file.write_all(output.as_bytes())?;
    }
    Ok(())
}
pub fn save_cookie_in_local(response: &reqwest::Response) -> Result<(), anyhow::Error> {
    let cookies_str = response
        .cookies()
        .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
        .join(";");
    let path = dirs::home_dir().unwrap().join(".cpm");
    //create $HOME/.cpm
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
pub fn username_and_password() -> (String, String) {
    println!("Please input Your username and password");
    let username = rpassword::prompt_password("Username > ").unwrap();
    let password = rpassword::prompt_password("Password > ").unwrap();
    (username, password)
}
pub fn local_cookie_headers() -> Result<HeaderMap, anyhow::Error> {
    let cookiejar_path = dirs::home_dir().unwrap().join(".cpm").join("cookie.jar");
    let file = std::fs::File::open(cookiejar_path)?;
    let reader = std::io::BufReader::new(file);

    let mut cookie_headers = HeaderMap::new();
    reader.lines().map(|line| line.unwrap()).for_each(|line| {
        cookie_headers.insert(COOKIE, HeaderValue::from_str(&line).unwrap());
    });
    Ok(cookie_headers)
}
