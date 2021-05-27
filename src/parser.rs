pub trait Parser {
    fn problem_name(&self) -> Option<String>;
    fn contest_name(&self) -> Option<String>;
    fn sample_cases(&self) -> Vec<(String, String)>;
}
