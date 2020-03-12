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
