package problem

import (
	"github.com/PuerkitoBio/goquery"
)

type Codeforces struct {
	URL string
	Doc *goquery.Document
}

func (c *Codeforces) newDocument() error {
	doc, err := goquery.NewDocument(c.URL)
	if err != nil {
		return err
	}
	c.Doc = doc
	return nil
}

func (c *Codeforces) GetProblemName() (string, error) {

	return "", nil
}
