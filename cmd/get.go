// Copyright © 2017 NAME HERE <EMAIL ADDRESS>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package cmd

import (
	"container/list"
	"fmt"
	"io/ioutil"
	"net/url"
	"os"
	"path/filepath"
	"strings"

	"github.com/spf13/viper"

	"github.com/spf13/cobra"
	"github.com/togatoga/cpm/problem"
)

// getCmd represents the get command
var getCmd = &cobra.Command{
	Use:   "get",
	Short: "Create problem directory by URL",
	Long: `Get problem information by URL.
Create structural problem directory under cpm root
Currently cpm supports Codeforces, AtCoder.`,
	Run: get,
}

func get(cmd *cobra.Command, args []string) {
	if len(args) == 0 {
		return
	}
	que := list.New()
	values := map[string]bool{}
	que.PushBack(args[0])
	for que.Len() > 0 {
		v := que.Remove(que.Front()).(string)
		if _, ok := values[v]; ok {
			continue
		}
		values[v] = true
		url, err := url.Parse(v)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
			continue
		}
		p, err := getProblem(url)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
			continue
		}
		if p.IsProblemPage() {
			if err := createProblemDir(p); err != nil {
				fmt.Printf("Error: %v\n", err)
				continue
			}
			sampleCases, err := p.GetSampleTestCase()
			if err != nil {
				fmt.Printf("Error: %v\n", err)
				continue
			}
			if err := createSampleFiles(p, sampleCases); err != nil {
				fmt.Printf("Error: %v\n", err)
				continue
			}

		} else if p.IsContestPage() {
			urlSet, err := p.GetProblemURLSet()
			if err != nil {
				fmt.Printf("Error: %v\n", err)
				continue
			}
			for _, url := range urlSet {
				que.PushBack(url)
			}
		}
	}
}
func sanitizeProblem(s string) string {
	//for Rust
	s = strings.Replace(s, ".", "_", -1)
	s = strings.Replace(s, "*", "", -1)
	return s
}

func sanitize(s string) string {
	s = strings.Replace(s, " ", "", -1)
	s = strings.Replace(s, "/", "\\", -1)
	return s
}

func getProblemDirPath(p problem.Problem) (string, error) {

	contestSiteName := p.GetContestSiteName()
	contestName, err := p.GetContestName()
	if err != nil {
		return "", err
	}
	contestProblem, err := p.GetProblemName()
	if err != nil {
		return "", err
	}
	root := viper.Get("root").(string)

	contestSiteName = sanitize(contestSiteName)
	contestName = sanitize(contestName)
	contestProblem = sanitizeProblem(sanitize(contestProblem))
	fmt.Println(contestSiteName, contestName, contestProblem)
	dir := filepath.Join(root, contestSiteName, contestName, contestProblem)

	return dir, nil
}
func createProblemDir(p problem.Problem) error {
	dir, err := getProblemDirPath(p)
	if err != nil {
		return err
	}
	if err := os.MkdirAll(dir, 0766); err != nil {
		return fmt.Errorf("Can not create directory: %v", err)
	}
	//json
	file := filepath.Join(dir, ".problem.json")
	f, err := os.Create(file)
	defer f.Close()
	if err != nil {
		return fmt.Errorf("Can not create file: %v", err)
	}

	fmt.Printf("Create directory %v\n", dir)
	return nil
}

func createSampleFiles(p problem.Problem, sampleCases []problem.TestCase) error {
	dir, err := getProblemDirPath(p)
	if err != nil {
		return err
	}
	sampleDir := filepath.Join(dir, "sample")
	if err := os.MkdirAll(sampleDir, 0766); err != nil {
		return fmt.Errorf("Can not create directory: %v", err)
	}
	n := len(sampleCases)
	for i := 0; i < n; i++ {
		input := sampleCases[i].Input
		output := sampleCases[i].Output
		fileName := fmt.Sprintf("sample_%02d", i)
		inFileName := filepath.Join(sampleDir, fileName+"_in.txt")
		outFileName := filepath.Join(sampleDir, fileName+"_out.txt")
		if err := ioutil.WriteFile(inFileName, []byte(input), 0644); err != nil {
			return fmt.Errorf("Can not create file: %v", err)
		}

		if err := ioutil.WriteFile(outFileName, []byte(output), 0644); err != nil {
			return fmt.Errorf("Can not create file: %v", err)
		}

	}
	fmt.Println("Create sample input/output files")
	return nil
}

func getProblem(url *url.URL) (problem.Problem, error) {
	host := url.Host
	switch host {
	case "codeforces.com":
		p, err := problem.NewCodeforces(url)
		if err != nil {
			return nil, err
		}
		return p, nil
	case "atcoder.jp":
		p, err := problem.NewAtCoder(url)
		if err != nil {
			return nil, err
		}
		return p, nil
	}

	return nil, fmt.Errorf("Can not parse this URL %s", url.String())
}

func init() {
	RootCmd.AddCommand(getCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// getCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// getCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
