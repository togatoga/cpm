use std::collections::BTreeMap;

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

        // try an English first
        let en_ms = en_pattern.matches(&self.html);
        let ja_input_cases = ja_input_pattern
            .matches(&self.html)
            .into_iter()
            .map(|m| (m["id"].to_string(), m["value"].to_string()))
            .collect::<Vec<_>>();
        let ja_id_to_output_case = ja_output_pattern
            .matches(&self.html)
            .into_iter()
            .map(|m| (m["id"].to_string(), m["value"].to_string()))
            .collect::<BTreeMap<String, String>>();
        let mut en_input_cases = vec![];
        let mut en_id_to_output_case = BTreeMap::default();
        for m in en_ms.iter() {
            match m["type"].as_str() {
                "Input" => {
                    en_input_cases.push((m["id"].to_string(), m["value"].to_string()));
                }
                "Output" => {
                    en_id_to_output_case.insert(m["id"].to_string(), m["value"].to_string());
                }

                _ => {
                    panic!("UNKNOWN type: {}", m["type"]);
                }
            };
        }
        if !ja_input_cases.is_empty() {
            ja_input_cases
                .into_iter()
                .map(|(id, input)| {
                    (
                        input,
                        ja_id_to_output_case
                            .get(&id)
                            .unwrap_or(&"".to_string())
                            .clone(),
                    )
                })
                .collect::<Vec<(String, String)>>()
        } else {
            en_input_cases
                .into_iter()
                .map(|(id, input)| {
                    (
                        input,
                        en_id_to_output_case
                            .get(&id)
                            .unwrap_or(&"".to_string())
                            .clone(),
                    )
                })
                .collect::<Vec<(String, String)>>()
        }
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

#[cfg(test)]
mod tests {

    use crate::parser::Parser;

    use super::AtCoderParser;

    async fn request(url: &str) -> String {
        let html = reqwest::get(url)
            .await
            .expect("failed to request")
            .text()
            .await
            .expect("failed to request");
        html
    }
    fn equal(samples: &[(String, String)], expecteds: &[(&str, &str)], url: &str) {
        let expecteds = expecteds
            .iter()
            .map(|x| (x.0.to_string(), x.1.to_string()))
            .collect::<Vec<_>>();

        for (i, (input, output)) in samples.iter().enumerate() {
            let (expected_input, expected_output) = &expecteds[i];
            assert!(
                (input, output) == (expected_input, expected_output),
                "URL: {} Case: {} {:?}:{:?}",
                url,
                i,
                (input, output),
                (expected_input, expected_output)
            );
        }
        assert!(samples.len() == expecteds.len());
    }
    async fn assert_sample_cases(url: &str, expecteds: &[(&str, &str)]) {
        let html = request(url).await;
        let parser = AtCoderParser::new(&html);
        let samples = parser.sample_cases();
        equal(&samples, expecteds, url);
    }

    #[tokio::test]
    async fn test_sample_cases() {
        let expecteds = vec![("2 3", "2"), ("3 4", "4"), ("3 6", "6")];
        assert_sample_cases(
            "https://atcoder.jp/contests/typical90/tasks/typical90_ag",
            &expecteds,
        )
        .await;

        let expecteds = vec![
            ("4 6", "12"),
            ("1000000000000000000 3", "Large"),
            ("1000000000000000000 1", "1000000000000000000"),
        ];
        assert_sample_cases(
            "https://atcoder.jp/contests/typical90/tasks/typical90_al",
            &expecteds,
        )
        .await;

        let expecteds = vec![
            ("5\n180 186 189 191 218", "Yes\n1 1\n2 3 4"),
            ("2\n123 523", "Yes\n1 1\n1 2"),
            ("6\n2013 1012 2765 2021 508 6971", "No"),
        ];
        assert_sample_cases(
            "https://atcoder.jp/contests/abc200/tasks/abc200_d",
            &expecteds,
        )
        .await;

        let expecteds = vec![("999 434", "2"), ("255 15", "2"), ("9999999999 1", "0")];
        assert_sample_cases(
            "https://atcoder.jp/contests/typical90/tasks/typical90_y",
            &expecteds,
        )
        .await;

        let expecteds = vec![
            ("3 3\n122\n131\n322", "2"),
            ("3 3\n111\n231\n321", "0"),
            ("4 5\n12334\n41123\n43214\n21344", "5"),
        ];
        assert_sample_cases(
            "https://atcoder.jp/contests/code-festival-2015-relay/tasks/cf_2015_relay_h",
            &expecteds,
        )
        .await;
    }
    #[tokio::test]
    async fn test_abc057_d() {
        assert_sample_cases("https://atcoder.jp/contests/abc057/tasks/abc057_d", 
        &[("5 2 2\n1 2 3 4 5", "4.500000\n1"),
            ("4 2 3\n10 20 10 10", "15.000000\n3"),
            ("5 1 5\n1000000000000000 999999999999999 999999999999998 999999999999997 999999999999996", "1000000000000000.000000\n1"),
            ("50 1 50\n1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1", "1.000000\n1125899906842623")]).await;
    }

    #[tokio::test]
    async fn test_abc161_e() {
        assert_sample_cases(
            "https://atcoder.jp/contests/abc161/tasks/abc161_e",
            &[
                ("11 3 2\nooxxxoxxxoo", "6"),
                ("5 2 3\nooxoo", "1\n5"),
                ("5 1 0\nooooo", ""),
                ("16 4 3\nooxxoxoxxxoxoxxo", "11\n16"),
            ],
        )
        .await;
    }
}
