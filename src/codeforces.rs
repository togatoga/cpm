use crate::parser::Parser;
pub struct CodeforcesParser {
    document: scraper::Html,
}

impl CodeforcesParser {
    pub fn new(html: &str) -> CodeforcesParser {
        CodeforcesParser {
            document: scraper::Html::parse_document(html),
        }
    }
}

impl Parser for CodeforcesParser {
    fn problem_name(&self) -> Option<String> {
        None
    }
    fn contest_name(&self) -> Option<String> {
        None
    }
    fn sample_cases(&self) -> Option<Vec<(String, String)>> {
        None
    }
}
