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
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"strings"

	"github.com/mitchellh/go-homedir"
	"github.com/spf13/cobra"
	"github.com/togatoga/cpm/problem"
)

// getCmd represents the get command
var getCmd = &cobra.Command{
	Use:   "get",
	Short: "A brief description of your command",
	Long: `A longer description that spans multiple lines and likely contains examples
and usage of using your command. For example:

Cobra is a CLI library for Go that empowers applications.
This application is a tool to generate the needed files
to quickly create a Cobra application.`,
	Run: func(cmd *cobra.Command, args []string) {
		for _, arg := range args {
			url, err := url.Parse(arg)
			if err != nil {
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
			}
		}
	},
}

func createProblemDir(p problem.Problem) error {
	dir, err := homedir.Dir()
	if err != nil {
		return err
	}
	contestSiteName := p.GetContestSiteName()
	contestName, err := p.GetContestName()
	if err != nil {
		return err
	}
	contestProblem, err := p.GetProblemName()
	if err != nil {
		return err
	}

	contestSiteName = strings.Replace(contestSiteName, " ", "", -1)
	contestName = strings.Replace(contestName, " ", "", -1)
	contestProblem = strings.Replace(contestProblem, " ", "", -1)

	dir = filepath.Join(dir, ".cpm", "src", contestSiteName, contestName, contestProblem)
	if err := os.MkdirAll(dir, 0766); err != nil {
		return fmt.Errorf("Can not create directory: %v", err)
	}
	file := filepath.Join(dir, ".problem")
	f, err := os.Create(file)
	defer f.Close()
	if err != nil {
		return fmt.Errorf("Can not create file: %v", err)
	}

	fmt.Printf("Create directory %v", dir)
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
	case "beta.atcoder.jp":
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
