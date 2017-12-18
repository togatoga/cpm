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
	"io/ioutil"
	"path/filepath"
	"strconv"

	homedir "github.com/mitchellh/go-homedir"
	"github.com/spf13/cobra"
)

// listCmd represents the list command
var listCmd = &cobra.Command{
	Use:   "list",
	Short: "A brief description of your command",
	Long: `A longer description that spans multiple lines and likely contains examples
and usage of using your command. For example:

Cobra is a CLI library for Go that empowers applications.
This application is a tool to generate the needed files
to quickly create a Cobra application.`,
	Run: func(cmd *cobra.Command, args []string) {
		problemDirs, err := getProblemDirs()
		if err != nil {
			return
		}
		for _, dir := range problemDirs {
			fmt.Println(strconv.Quote(dir))
		}
	},
}

func getProblemDirs() ([]string, error) {
	dir, err := homedir.Dir()
	if err != nil {
		return nil, err
	}
	baseDir := filepath.Join(dir, ".cpm", "src")
	sites, err := ioutil.ReadDir(baseDir)
	if err != nil {
		return nil, err
	}
	var problemDirs []string

	for _, site := range sites {
		if site.IsDir() {
			contests, err := ioutil.ReadDir(filepath.Join(baseDir, site.Name()))
			if err != nil {
				return nil, err
			}
			for _, contest := range contests {
				if contest.IsDir() {
					problems, err := ioutil.ReadDir(filepath.Join(baseDir, site.Name(), contest.Name()))
					if err != nil {
						return nil, err
					}
					for _, problem := range problems {
						if err != nil {
							return nil, err
						}
						problemDir := filepath.Join(baseDir, site.Name(), contest.Name(), problem.Name())
						problemDirs = append(problemDirs, problemDir)
					}
				}
			}
		}
	}
	return problemDirs, nil
}

func init() {
	RootCmd.AddCommand(listCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// listCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// listCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
