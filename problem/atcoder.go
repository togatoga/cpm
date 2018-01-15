package problem

import (
	"fmt"
	"net/http"
	"net/http/cookiejar"
	"net/url"
	"os"
	"strings"

	"github.com/PuerkitoBio/goquery"
)

type AtCoder struct {
	URL *url.URL
	Doc *goquery.Document
}

func NewAtCoder(URL *url.URL) (*AtCoder, error) {
	c := new(AtCoder)
	c.URL = URL
	err := c.newDocument()
	if err != nil {
		return nil, err
	}
	return c, nil
}

func (c *AtCoder) GetContestSiteName() string {
	url := c.URL
	return url.Host
}

func (c *AtCoder) newDocument() error {
	jar, err := cookiejar.New(nil)
	if err != nil {
		return err
	}
	var cookies []*http.Cookie
	cookie := &http.Cookie{
		Name:  "REVEL_SESSION",
		Value: os.Getenv("ATCODER_SESSION"),
	}
	cookies = append(cookies, cookie)
	jar.SetCookies(c.URL, cookies)

	client := &http.Client{
		Jar: jar,
	}
	resp, err := client.Get(c.URL.String())
	if err != nil {
		return err
	}

	doc, err := goquery.NewDocumentFromResponse(resp)
	if err != nil {
		return err
	}
	c.Doc = doc

	return nil
}
func (c *AtCoder) GetContestName() (string, error) {
	doc := c.Doc
	s := doc.Find(".contest-title").First()
	if s.Text() == "" {
		return "", fmt.Errorf("Can not find Contest Name")
	}
	return s.Text(), nil
}
func (c *AtCoder) GetProblemName() (string, error) {
	doc := c.Doc
	s := doc.Find("head > title").First()
	if s.Text() == "" {
		return "", fmt.Errorf("Can not find Problem Name")
	}
	return s.Text(), nil
}

func (c *AtCoder) GetTimeLimit() (string, error) {
	return "", nil
}
func (c *AtCoder) GetMemoryLimit() (string, error) {
	return "", nil
}

func (c *AtCoder) GetSampleTestCase() ([]TestCase, error) {
	doc := c.Doc
	inputs := []string{}
	outputs := []string{}
	doc.Find("div#task-statement > span.lang > span.lang-ja > div.part > section > pre").Each(func(i int, s *goquery.Selection) {
		if s.Text() == "" {
			return
		}
		if i%2 == 0 {
			inputs = append(inputs, s.Text())
		} else {
			outputs = append(outputs, s.Text())
		}
	})
	n := len(inputs)
	if len(inputs) != len(outputs) || len(inputs) == 0 {
		return nil, fmt.Errorf("Can not get SampleTestCase")
	}

	testCases := []TestCase{}
	for i := 0; i < n; i++ {
		testCases = append(testCases, TestCase{Input: inputs[i], Output: outputs[i]})
	}
	return testCases, nil
}

func (c *AtCoder) GetProblemURLSet() ([]string, error) {
	doc := c.Doc
	urlSet := []string{}
	doc.Find("tbody > tr").Each(func(i int, s *goquery.Selection) {
		url, ok := s.Find("td > a").First().Attr("href")
		if ok {
			urlSet = append(urlSet, c.URL.Scheme+"://"+c.URL.Host+url)
		}
	})
	if len(urlSet) == 0 {
		return nil, fmt.Errorf("urlSet is empty")
	}
	return urlSet, nil
}

func (c *AtCoder) IsContestPage() bool {
	url := c.URL
	p := strings.Split(url.Path, "/")[1:]
	n := len(p)
	if n == 3 && p[n-1] == "tasks" {
		return true
	}
	return false
}

func (c *AtCoder) IsProblemPage() bool {
	url := c.URL
	p := strings.Split(url.Path, "/")[1:]
	n := len(p)

	if n == 4 && p[n-2] == "tasks" && p[n-1] != "" {
		return true
	}
	return false
}
