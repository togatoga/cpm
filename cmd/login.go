/*
Copyright © 2019 NAME HERE <EMAIL ADDRESS>

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/
package cmd

import (
	"bufio"
	"fmt"
	"net/http/cookiejar"
	"net/url"
	"os"
	"strings"
	"syscall"

	"github.com/k0kubun/pp"
	"github.com/togatoga/cpm/problem"
	"golang.org/x/crypto/ssh/terminal"

	"github.com/spf13/cobra"
)

func credentials() (string, string) {
	reader := bufio.NewReader(os.Stdin)

	fmt.Print("Enter Username: ")
	username, _ := reader.ReadString('\n')

	fmt.Print("Enter Password: ")
	bytePassword, _ := terminal.ReadPassword(int(syscall.Stdin))

	password := string(bytePassword)

	return strings.TrimSpace(username), strings.TrimSpace(password)
}

// loginCmd represents the login command
var loginCmd = &cobra.Command{
	Use:   "login",
	Short: "Login services and save your cookies in your local",
	Long: `A longer description that spans multiple lines and likely contains examples
and usage of using your command. For example:

Cobra is a CLI library for Go that empowers applications.
This application is a tool to generate the needed files
to quickly create a Cobra application.`,
	Run: func(cmd *cobra.Command, args []string) {
		u, err := url.Parse("https://atcoder.jp/login")
		if err != nil {
			fmt.Printf("%s: %s", err, u)
		}
		username, password := credentials()

		c := problem.NewAtCoder(u)
		err = c.MakeGetRequest()
		if err != nil {
			fmt.Printf("%s", err)
		}
		c.ParseResponse()
		token, _ := c.Doc.Find("input[name='csrf_token']").Attr("value")

		values := url.Values{
			"username":   {username},
			"password":   {password},
			"csrf_token": {token},
		}
		jar, err := cookiejar.New(nil)
		if err != nil {
			fmt.Printf("%s", err)
		}
		jar.SetCookies(u, c.Resp.Cookies())
		err = c.MakePostFormRequest(values, jar)
		if err != nil {
			fmt.Printf("%s", err)
		}
		err = c.ParseResponse()
		if err != nil {
			fmt.Printf("%s", err)
		}

		html, _ := c.Doc.Html()

		pp.Println(html)
	},
}

func init() {
	RootCmd.AddCommand(loginCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// loginCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// loginCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
