use crate::parser::Parser;
use easy_scraper::Pattern;
use itertools::Itertools;
use scraper::Selector;

pub struct CodeforcesParser {
    document: String,
}

impl CodeforcesParser {
    pub fn new(html: &str) -> CodeforcesParser {
        CodeforcesParser {
            document: html.to_string(),
        }
    }
}

impl CodeforcesParser {
    pub fn problem_url_list(&self, path: &str) -> Vec<String> {
        let document = scraper::Html::parse_document(&self.document);
        let selector = Selector::parse("a").expect("invalid selector");

        let mut problem_url_list = document
            .select(&selector)
            .filter_map(|element| element.value().attr("href"))
            // e.g /contest/1234/problem/A
            .filter(|url| url.starts_with(path) && url.split('/').contains(&"problem"))
            .map(|url| url.to_string())
            .collect::<Vec<_>>();
        problem_url_list.sort();
        problem_url_list.dedup();
        problem_url_list
    }
}

impl Parser for CodeforcesParser {
    fn problem_name(&self) -> Option<String> {
        let pattern = Pattern::new(
            r#"
        <div class="problem-statement">
        <div class="header">
        <div class="title">{{problem_name}}
        </div>
        </div>
        </div>
        "#,
        )
        .unwrap();
        let ms = pattern.matches(&self.document);
        ms.first()
            .map(|problem_name| problem_name["problem_name"].to_string())
    }
    fn contest_name(&self) -> Option<String> {
        let pattern = Pattern::new(
            r#"
            <table class="rtable ">
            <tbody>
                <tr>
                   <th class="left" style="width:100%;"><a style="color: black" href={{}}>{{contest_name}}</a></th>
                </tr>
            </tbody>
        </table>
         "#
        ).unwrap();
        let ms = pattern.matches(&self.document);
        ms.first()
            .map(|contest_name| contest_name["contest_name"].to_string())
    }
    fn sample_cases(&self) -> Vec<(String, String)> {
        let document = scraper::Html::parse_document(&self.document);
        let sample_test_selector = scraper::Selector::parse(r#"div[class="sample-test"]"#).unwrap();
        let input_selector = scraper::Selector::parse(r#"div[class="input"]"#).unwrap();
        let output_selector = scraper::Selector::parse(r#"div[class="output"]"#).unwrap();
        let pre_selector = scraper::Selector::parse("pre").unwrap();

        let (inputs, outputs) = document
            .select(&sample_test_selector)
            .next()
            .map(|sample| {
                let sample_inputs = sample
                    .select(&input_selector)
                    .into_iter()
                    .filter_map(|input| {
                        input.select(&pre_selector).next().and_then(|pre| {
                            let sample_input = pre.text().into_iter().join("\n");
                            if sample_input.is_empty() {
                                None
                            } else {
                                Some(sample_input)
                            }
                        })
                    })
                    .collect::<Vec<String>>();
                let sample_outputs = sample
                    .select(&output_selector)
                    .into_iter()
                    .filter_map(|output| {
                        output.select(&pre_selector).next().and_then(|pre| {
                            let sample_output = pre.text().into_iter().join("\n");
                            if sample_output.is_empty() {
                                None
                            } else {
                                Some(sample_output)
                            }
                        })
                    })
                    .collect::<Vec<String>>();
                (sample_inputs, sample_outputs)
            })
            .unwrap_or((vec![], vec![]));

        inputs.into_iter().zip(outputs).collect()
    }
}
