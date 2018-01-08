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
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/fatih/color"

	pipline "github.com/mattn/go-pipeline"
	"github.com/spf13/cobra"
	"github.com/togatoga/cpm/problem"
)

// runCmd represents the run command
var runCmd = &cobra.Command{
	Use:   "run",
	Short: "A brief description of your command",
	Long: `A longer description that spans multiple lines and likely contains examples
and usage of using your command. For example:

Cobra is a CLI library for Go that empowers applications.
This application is a tool to generate the needed files
to quickly create a Cobra application.`,
	Run: run,
}

func run(cmd *cobra.Command, args []string) {
	if len(args) == 0 {
		return
	}
	execCmd := strings.Join(args, " ")
	testFiles, err := getTestFiles()
	if err != nil {
		fmt.Printf("Error %v:\n", err)
		return
	}
	acNum := 0
	testNum := 0
	fmt.Println("RUNNING TEST CASES...")
	for _, testFile := range testFiles {
		output, err := execTest(execCmd, testFile)
		if err != nil {
			fmt.Printf("Error: %v:\n", err)
		}
		ac, err := showResult(output, testFile)
		if err != nil {
			fmt.Printf("Error %v:\n", err)
		}
		testNum++
		if ac {
			acNum++
		}

	}
	fmt.Println("----------------------THE TEST RESULT----------------------")
	fmt.Printf("The test result is %d / %d\n", acNum, testNum)
}

func showResult(execOutput string, testFile problem.TestFile) (bool, error) {
	fmt.Println("-----------------------------------------")
	fmt.Printf("Name: %s\n", testFile.Name)
	fmt.Printf("Input: %s\n", filepath.Base(testFile.InputFile))
	fmt.Printf("Output: %s\n", filepath.Base(testFile.OutputFile))

	data, err := ioutil.ReadFile(testFile.OutputFile)
	if err != nil {
		return false, err
	}
	output := string(data)
	if execOutput == output {
		color.Green("[OK]\n")
		return true, nil
	}
	color.Yellow("[Wrong Answer]\n")
	fmt.Println("The output is", execOutput)
	fmt.Println("The judge output is", output)
	return false, nil
}

func getTestFiles() ([]problem.TestFile, error) {
	dir, err := os.Getwd()

	if err != nil {
		return nil, err
	}
	inputFiles := map[string]string{}
	outputFiles := map[string]string{}

	err = filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if !info.IsDir() {
			fileName := info.Name()
			p := filepath.Dir(path)
			//input
			if strings.HasSuffix(fileName, "_in.txt") == true {
				name := strings.TrimRight(fileName, "_in.txt")
				inputFiles[name] = filepath.Join(p, fileName)
			} else if strings.HasSuffix(fileName, "_out.txt") == true {
				name := strings.TrimRight(fileName, "_out.txt")
				outputFiles[name] = filepath.Join(p, fileName)
			}
		}
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("Fail to walk dir: %v", err)
	}
	var testFiles []problem.TestFile
	for name, inputFile := range inputFiles {
		outputFile, ok := outputFiles[name]
		if !ok {
			continue
		}
		testFiles = append(testFiles, problem.TestFile{Name: name, InputFile: inputFile, OutputFile: outputFile})
	}
	sort.Slice(testFiles, func(i, j int) bool {
		if strings.HasSuffix(testFiles[i].Name, "sample") && !strings.HasSuffix(testFiles[j].Name, "sample") {
			return true
		}
		if !strings.HasSuffix(testFiles[i].Name, "sample") && strings.HasSuffix(testFiles[j].Name, "sample") {
			return false
		}
		return testFiles[i].Name < testFiles[j].Name
	})
	return testFiles, nil
}

func execTest(execCmd string, testCase problem.TestFile) (string, error) {

	out, err := pipline.Output(
		[]string{"cat", testCase.InputFile},
		[]string{"sh", "-c", execCmd},
	)
	if err != nil {
		return "", err
	}
	return string(out), nil
}

func init() {
	RootCmd.AddCommand(runCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// runCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// runCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
