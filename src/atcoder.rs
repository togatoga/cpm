use crate::parser::Parser;
use easy_scraper::Pattern;
use itertools::Itertools;

pub struct AtCoderParser {
    html: String,
    document: scraper::Html,
}

impl Parser for AtCoderParser {
    fn problem_name(&self) -> Option<String> {
        let title_selector = scraper::Selector::parse("head > title").unwrap();
        let problem_name = self
            .document
            .select(&title_selector)
            .next()
            .map(|title| title.text().collect::<String>());
        problem_name
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
        let mut input_cases = vec![];
        let mut output_cases = vec![];

        let pattern = Pattern::new(
            r#"
            <div class="part">
            <section>
            <h3>Sample {{type}} {{id}}</h3><pre>
            {{value}}
            </pre>
            </section>
            </div>
            "#,
        )
        .unwrap();
        let ms = pattern.matches(&self.html);
        for m in ms.iter() {
            match m["type"].as_str() {
                "Input" => input_cases.push(m["value"].to_string()),
                "Output" => output_cases.push(m["value"].to_string()),
                _ => {
                    panic!("type must be Input or Output");
                }
            };
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
}

impl AtCoderParser {
    pub fn new(html: &str) -> AtCoderParser {
        AtCoderParser {
            html: html.to_string(),
            document: scraper::Html::parse_document(html),
        }
    }
    pub fn problem_url_list(&self) -> Option<Vec<String>> {
        //This function is supposed to be called from task url.
        //e.g https://atcoder.jp/contests/abc155/tasks

        let mut url_list = vec![];
        let main_container_selector =
            scraper::Selector::parse(r#"div[id="main-container"]"#).unwrap();
        if let Some(main_container) = self.document.select(&main_container_selector).next() {
            for a in main_container.select(&scraper::Selector::parse("a").unwrap()) {
                if let Some(url) = a.value().attr("href") {
                    url_list.push(url);
                }
            }
        }
        url_list.sort_unstable();
        url_list.dedup();
        let url_list: Vec<String> = url_list
            .iter()
            .filter_map(|url| {
                let paths: Vec<&str> = url.split('/').collect();
                if paths.len() == 5 {
                    // /contests/abc147/tasks/abc147_a

                    if paths[1] == "contests" && paths[3] == "tasks" {
                        return Some(url.to_string());
                    }
                }
                None
            })
            .collect();
        if !url_list.is_empty() {
            Some(url_list)
        } else {
            None
        }
    }

    pub fn csrf_token(&self) -> Option<String> {
        let selector = scraper::Selector::parse(r#"input[name="csrf_token"]"#).unwrap();
        if let Some(element) = self.document.select(&selector).next() {
            if let Some(token) = element.value().attr("value") {
                return Some(token.to_string());
            }
        }
        None
    }
}
