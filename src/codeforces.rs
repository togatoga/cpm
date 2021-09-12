use crate::parser::Parser;
use easy_scraper::Pattern;

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
        let mut sample_inputs = vec![];
        let mut sample_outputs = vec![];
        if let Some(sample) = document.select(&sample_test_selector).next() {
            let input_selector = scraper::Selector::parse(r#"div[class="input"]"#).unwrap();
            let output_selector = scraper::Selector::parse(r#"div[class="output"]"#).unwrap();

            for input in sample.select(&input_selector).into_iter() {
                let pre_selector = scraper::Selector::parse("pre").unwrap();
                if let Some(pre) = input.select(&pre_selector).next() {
                    let sample_input = pre.text().collect::<String>();
                    if !sample_input.is_empty() {
                        sample_inputs.push(sample_input);
                    }
                }
            }
            for output in sample.select(&output_selector).into_iter() {
                let pre_selector = scraper::Selector::parse("pre").unwrap();
                if let Some(pre) = output.select(&pre_selector).next() {
                    let sample_output = pre.text().collect::<String>();
                    if !sample_output.is_empty() {
                        sample_outputs.push(sample_output);
                    }
                }
            }
        }
        let samples: Vec<(String, String)> =
            sample_inputs.into_iter().zip(sample_outputs).collect();
        samples
    }
}
