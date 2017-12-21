package problem

import (
	"fmt"
	"net/url"
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
	doc, err := goquery.NewDocument(c.URL.String())
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
func (c *AtCoder) GetSampleInputs() ([]string, error) {
	return []string{}, nil
}
func (c *AtCoder) GetSampleOutpus() ([]string, error) {
	return []string{}, nil
}

func (c *AtCoder) IsContestPage() bool {
	return true
}

func (c *AtCoder) IsProblemPage() bool {
	url := c.URL
	p := strings.Split(url.Path, "/")
	n := len(p)

	if n == 5 && p[n-2] == "tasks" && p[n-1] != "" {
		return true
	}
	return false
}
