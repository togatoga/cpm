use itertools::Itertools;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use std::io::{BufRead, Write};

pub fn create_sample_test_files(test_cases: &[(String, String)]) -> Result<(), failure::Error> {
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
pub fn save_cookie_in_local(response: &reqwest::Response) -> Result<(), failure::Error> {
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
pub fn username_and_password() -> (String, String) {
    println!("Please input Your username and password");
    let username = rpassword::read_password_from_tty(Some("Username > ")).unwrap();
    let password = rpassword::read_password_from_tty(Some("Password > ")).unwrap();
    (username, password)
}
pub fn local_cookie_headers() -> Result<HeaderMap, failure::Error> {
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
