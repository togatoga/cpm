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
	"fmt"
	"net/url"

	"github.com/sirupsen/logrus"
	"github.com/togatoga/cpm/problem"

	"github.com/spf13/cobra"
)

// loginCmd represents the login command
var loginCmd = &cobra.Command{
	Use:   "login",
	Short: "Login services and save/update your cookies in your local",
	Long:  `Login services and save/update your cookies in your local`,
	Run: func(cmd *cobra.Command, args []string) {
		u, err := url.Parse("https://atcoder.jp/login")
		if err != nil {
			logrus.Fatal(err)

		}
		c := problem.NewAtCoder(u)
		err = c.Login()
		if err != nil {
			logrus.Fatal(err)
		}
		err = c.ParseResponse()
		if err != nil {
			logrus.Fatal(err)
		}
		for _, cookie := range c.Resp.Cookies() {
			fmt.Println(cookie)
		}
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
