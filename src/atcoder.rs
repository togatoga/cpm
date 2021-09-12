use crate::parser::Parser;
use easy_scraper::Pattern;
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

    fn sample_cases(&self) -> Vec<(String, String)> {
        let sample_cases = self.extract_sample_cases();
        if !sample_cases.is_empty() {
            sample_cases
        } else {
            self.extract_old_format_sample_cases()
                .map_or(vec![], |sample_cases| sample_cases)
        }
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
    fn extract_sample_cases(&self) -> Vec<(String, String)> {
        // new format
        let en_pattern = Pattern::new(
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
        let ja_input_pattern = Pattern::new(
            r#"
    <div class="part">
    <section>
    <h3>入力例 {{id}}</h3><pre>
    {{value}}
    </pre>
    </section>
    </div>
    "#,
        )
        .unwrap();
        let ja_output_pattern = Pattern::new(
            r#"
    <div class="part">
    <section>
    <h3>出力例 {{id}}</h3><pre>
    {{value}}
    </pre>
    </section>
    </div>
    "#,
        )
        .unwrap();

        let mut input_cases = vec![];
        let mut output_cases = vec![];
        // try an English first
        let en_ms = en_pattern.matches(&self.html);
        if en_ms.is_empty() {
            ja_input_pattern
                .matches(&self.html)
                .into_iter()
                .for_each(|m| input_cases.push(m["value"].to_string()));
            ja_output_pattern
                .matches(&self.html)
                .into_iter()
                .for_each(|m| output_cases.push(m["value"].to_string()));
        } else {
            for m in en_ms.iter() {
                match m["type"].as_str() {
                    "Input" => input_cases.push(m["value"].to_string()),
                    "Output" => output_cases.push(m["value"].to_string()),
                    _ => {
                        panic!("UNKNOWN type: {}", m["type"]);
                    }
                };
            }
        }
        input_cases
            .into_iter()
            .zip(output_cases)
            .map(|(input, output)| (input, output))
            .collect()
    }

    fn extract_old_format_sample_cases(&self) -> Option<Vec<(String, String)>> {
        let sample_case_patterns = || -> Vec<(Pattern, Pattern)> {
            let mut patterns = vec![];
            let en_input = Pattern::new(
                r#"
                <div class="part"><h3>Sample Input {{id}}</h3><section><pre>
            {{value}}
            </pre></section></div>
            "#,
            )
            .unwrap();
            let en_output = Pattern::new(
                r#"
            <div class="part"><h3>Sample Output {{id}}</h3><section><pre>
            {{value}}
            </pre></section></div>
            "#,
            )
            .unwrap();
            patterns.push((en_input, en_output));

            let ja_input = Pattern::new(
                r#"
                <div class="part"><h3>入力例{{id}}</h3><section><pre>
            {{value}}
            </pre></section></div>
            "#,
            )
            .unwrap();
            let ja_output = Pattern::new(
                r#"
            <div class="part"><h3>出力例{{id}}</h3><section><pre>
            {{value}}
            </pre></section></div>
            "#,
            )
            .unwrap();
            patterns.push((ja_input, ja_output));

            let ja_input = Pattern::new(
                r#"
                <div class="part">
                <section>
                <h3>入力例{{id}}</h3>
                <pre>
                {{value}}
                </pre>

                </section>
                </div>
            "#,
            )
            .unwrap();
            let ja_output = Pattern::new(
                r#"
                <h3>出力例{{id}}</h3>
                <pre>
                {{value}}
                </pre>                
            "#,
            )
            .unwrap();
            patterns.push((ja_input, ja_output));

            patterns
        };

        for (input_pattern, output_pattern) in sample_case_patterns() {
            let input_matches = input_pattern.matches(&self.html);
            let output_matches = output_pattern.matches(&self.html);
            //dbg!(&input_matches, &output_matches);
            let matches: Vec<_> = input_matches
                .into_iter()
                .zip(output_matches)
                .map(|(input, output)| (input["value"].to_string(), output["value"].to_string()))
                .collect();
            if !matches.is_empty() {
                return Some(matches);
            }
        }

        None
    }
}
