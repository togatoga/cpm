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
        if let Some(problem_name) = ms.first() {
            Some(problem_name["problem_name"].to_string())
        } else {
            None
        }
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
        if let Some(contest_name) = ms.first() {
            Some(contest_name["contest_name"].to_string())
        } else {
            None
        }
    }
    fn sample_cases(&self) -> Option<Vec<(String, String)>> {
        let document = scraper::Html::parse_document(&self.document);
        let sample_test_selector = scraper::Selector::parse(r#"div[class="sample-test"]"#).unwrap();
        if let Some(sample) = document.select(&sample_test_selector).next() {}
        None
    }
}
