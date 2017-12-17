package problem

import (
	"fmt"

	"github.com/PuerkitoBio/goquery"
)

type Codeforces struct {
	URL string
	Doc *goquery.Document
}

func NewCodeforces(URL string) (*Codeforces, error) {
	c := new(Codeforces)
	c.URL = URL
	err := c.newDocument()
	if err != nil {
		return nil, err
	}
	return c, nil
}
func (c *Codeforces) newDocument() error {
	doc, err := goquery.NewDocument(c.URL)
	if err != nil {
		return err
	}
	c.Doc = doc
	return nil
}
func (c *Codeforces) GetContestName() (string, error) {
	doc := c.Doc
	name, ok := doc.Find("span > a").First().Attr("title")
	if !ok {
		return "", fmt.Errorf("Can not find Contest Name")
	}
	return name, nil
}
func (c *Codeforces) GetProblemName() (string, error) {
	doc := c.Doc
	s := doc.Find("div.problem-statement div.header div.title").First()
	if s.Text() == "" {
		return "", fmt.Errorf("Can not find Problem Name")
	}
	return s.Text(), nil
}
