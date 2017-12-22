package problem

import (
	"fmt"
	"net/url"

	"github.com/PuerkitoBio/goquery"
)

type Codeforces struct {
	URL *url.URL
	Doc *goquery.Document
}

func NewCodeforces(URL *url.URL) (*Codeforces, error) {
	c := new(Codeforces)
	c.URL = URL
	err := c.newDocument()
	if err != nil {
		return nil, err
	}
	return c, nil
}
func (c *Codeforces) newDocument() error {
	doc, err := goquery.NewDocument(c.URL.String())
	if err != nil {
		return err
	}
	c.Doc = doc
	return nil
}
func (c *Codeforces) GetContestSiteName() string {
	url := c.URL
	return url.Host
}

func (c *Codeforces) GetContestName() (string, error) {
	doc := c.Doc
	name := doc.Find("div > table.rtable > tbody > tr > th.left > a").First()

	if name.Text() == "" {
		return "", fmt.Errorf("Can not find Contest Name")
	}

	return name.Text(), nil
}
func (c *Codeforces) GetProblemName() (string, error) {
	doc := c.Doc
	s := doc.Find("div.problem-statement div.header div.title").First()

	if s.Text() == "" {
		return "", fmt.Errorf("Can not find Problem Name")
	}
	return s.Text(), nil
}
func (c *Codeforces) GetTimeLimit() (string, error) {
	return "", nil
}
func (c *Codeforces) GetMemoryLimit() (string, error) {
	return "", nil
}
func (c *Codeforces) GetSampleTestCase() ([]TestCase, error) {
	return nil, nil
}

func (c *Codeforces) GetProblemURLSet() ([]string, error) {
	doc := c.Doc
	urlSet := []string{}
	doc.Find("td.id").Each(func(i int, s *goquery.Selection) {
		url, ok := s.Find("a").First().Attr("href")
		if ok {
			urlSet = append(urlSet, c.URL.Scheme+"://"+c.URL.Host+url)
		}
	})

	return urlSet, nil
}
func (c *Codeforces) IsContestPage() bool {
	urlSet, err := c.GetProblemURLSet()
	if err != nil || len(urlSet) == 0 {
		return false
	}
	return true
}

func (c *Codeforces) IsProblemPage() bool {
	contestName, err := c.GetContestName()
	if err != nil || contestName == "" {
		return false
	}
	problemName, err := c.GetProblemName()
	if err != nil || problemName == "" {
		return false
	}
	return true
}
